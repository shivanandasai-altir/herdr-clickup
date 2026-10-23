//! A task that spans repos: one folder, one worktree per repo, one branch, one agent.
use crate::hook::RepoWorktree;
use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// `~/.herdr/tasks/<branch>`: where a multi-repo task lives.
pub fn task_dir(branch: &str) -> Result<PathBuf> {
    let home = env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".herdr")
        .join("tasks")
        .join(branch))
}

/// The last path segment of a repo, used as its folder name.
pub fn repo_name(repo: &str) -> String {
    Path::new(repo)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "repo".into())
}

fn same_repo(a: &str, b: &str) -> bool {
    let canon = |p: &str| fs::canonicalize(p).unwrap_or_else(|_| PathBuf::from(p));
    canon(a) == canon(b)
}

/// A folder name no existing worktree uses: `webui`, then `webui-2`, and so on.
pub fn unique_name(repo: &str, taken: &[RepoWorktree]) -> String {
    let base = repo_name(repo);
    let mut name = base.clone();
    let mut n = 2;
    while taken.iter().any(|w| {
        Path::new(&w.path)
            .file_name()
            .is_some_and(|f| f.to_string_lossy() == name)
    }) {
        name = format!("{base}-{n}");
        n += 1;
    }
    name
}

fn git_output(repo: &str, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .with_context(|| format!("running git in {repo}"))
}

fn git_ok(repo: &str, args: &[&str]) -> bool {
    git_output(repo, args).is_ok_and(|o| o.status.success())
}

fn git(repo: &str, args: &[&str]) -> Result<()> {
    let out = git_output(repo, args)?;
    if !out.status.success() {
        bail!(
            "git {} in {repo}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// The base a worktree starts from: the remote default branch, else the repo's current HEAD.
pub fn base_ref(repo: &str) -> String {
    if git_ok(repo, &["rev-parse", "--verify", "--quiet", "origin/HEAD"]) {
        "origin/HEAD".into()
    } else {
        "HEAD".into()
    }
}

/// Adds a worktree of each repo under `dir` on `branch`, skipping repos already there. Returns only
/// the new worktrees, so a caller can grow the link.
pub fn add_worktrees(
    dir: &Path,
    repos: &[String],
    branch: &str,
    existing: &[RepoWorktree],
) -> Result<Vec<RepoWorktree>> {
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let mut all: Vec<RepoWorktree> = existing.to_vec();
    let mut added = Vec::new();
    for repo in repos {
        if all.iter().any(|w| same_repo(&w.repo, repo)) {
            continue;
        }
        let path = dir.join(unique_name(repo, &all));
        let base = base_ref(repo);
        let target = path.display().to_string();
        git(repo, &["worktree", "add", "-b", branch, &target, &base])
            .with_context(|| format!("creating a worktree of {repo} for {branch}"))?;
        let worktree = RepoWorktree {
            repo: repo.clone(),
            path: target,
        };
        all.push(worktree.clone());
        added.push(worktree);
    }
    Ok(added)
}

/// Removes the worktrees, even when they hold uncommitted changes. The caller asked first.
pub fn remove_worktrees(worktrees: &[RepoWorktree]) -> Vec<String> {
    let mut failed = Vec::new();
    for w in worktrees {
        // Already removed by an earlier attempt: nothing left to do.
        if !Path::new(&w.path).exists() {
            continue;
        }
        if let Err(e) = git(&w.repo, &["worktree", "remove", "--force", &w.path]) {
            failed.push(format!("{}: {e:#}", w.repo));
        }
    }
    failed
}

/// Repos with uncommitted or untracked changes.
pub fn dirty(worktrees: &[RepoWorktree]) -> Vec<String> {
    worktrees
        .iter()
        .filter(|w| {
            git_output(&w.path, &["status", "--porcelain"])
                .is_ok_and(|o| o.status.success() && !o.stdout.is_empty())
        })
        .map(|w| w.repo.clone())
        .collect()
}

/// The instructions a multi-repo agent works from: same branch everywhere, one PR per repo.
pub fn write_agents_md(
    dir: &Path,
    title: &str,
    url: &str,
    branch: &str,
    worktrees: &[RepoWorktree],
    agent: &str,
) -> Result<()> {
    let mut md = format!("# {title}\n\nClickUp task: {url}\nBranch: `{branch}`\n\n");
    md += "Work in the repo folders below. Every repo uses the same branch name. ";
    md += "Open one pull request per repo, and link the PRs to each other and to the ClickUp task.\n\n## Repos\n\n";
    for w in worktrees {
        md += &format!("- `{}`\n", w.path);
    }
    fs::write(dir.join("AGENTS.md"), &md)
        .with_context(|| format!("writing {}/AGENTS.md", dir.display()))?;
    // Claude reads CLAUDE.md, not AGENTS.md.
    if agent == "claude" {
        fs::write(dir.join("CLAUDE.md"), &md)
            .with_context(|| format!("writing {}/CLAUDE.md", dir.display()))?;
    }
    Ok(())
}

/// The repo list of a spec change, for the user to review and edit before implementation.
pub fn write_repos(change_dir: &Path, repos: &[String]) -> Result<()> {
    let mut text =
        String::from("# Absolute paths of the repos this change touches, one per line.\n");
    for r in repos {
        text.push_str(r);
        text.push('\n');
    }
    fs::write(change_dir.join("repos.txt"), text)
        .with_context(|| format!("writing {}/repos.txt", change_dir.display()))
}

/// The repo paths in a change's `repos.txt`; a missing file means no extra repos.
pub fn read_repos(change_dir: &Path) -> Result<Vec<String>> {
    let path = change_dir.join("repos.txt");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    let mut out: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !Path::new(line).is_absolute() {
            bail!("{}: repo path is not absolute: {line}", path.display());
        }
        if !out.iter().any(|r| same_repo(r, line)) {
            out.push(line.to_string());
        }
    }
    Ok(out)
}

/// Adds a repo to a change's `repos.txt`, so implementation keeps it.
pub fn add_repo(change_dir: &Path, repo: &str) -> Result<()> {
    let mut repos = read_repos(change_dir)?;
    if repos.iter().any(|r| same_repo(r, repo)) {
        return Ok(());
    }
    repos.push(repo.to_string());
    write_repos(change_dir, &repos)
}
