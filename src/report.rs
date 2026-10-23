//! What the agent did, for the ClickUp draft the hook proposes.
use crate::hook::Link;
use std::{fs, path::PathBuf, process::Command};

const MAX_WHY: usize = 500;
const MAX_COMMITS: usize = 10;

/// A short summary of the link's work: the spec's proposal and tasks, or the code's commits and files.
pub fn work_summary(link: &Link) -> String {
    match link.change.as_deref() {
        Some(change) => spec_summary(link, change),
        None => code_summary(link),
    }
}

fn host(link: &Link) -> Option<PathBuf> {
    link.worktrees
        .first()
        .map(|w| PathBuf::from(&w.path))
        .or_else(|| link.dir.as_ref().map(PathBuf::from))
}

fn spec_summary(link: &Link, change: &str) -> String {
    let Some(host) = host(link) else {
        return String::new();
    };
    let dir = host.join("openspec").join("changes").join(change);
    let why = fs::read_to_string(dir.join("proposal.md"))
        .ok()
        .map(|text| why_section(&text))
        .unwrap_or_default();
    let (done, total) = fs::read_to_string(dir.join("tasks.md"))
        .ok()
        .map(|text| task_counts(&text))
        .unwrap_or((0, 0));
    spec_text(&why, done, total)
}

fn code_summary(link: &Link) -> String {
    let worktrees: Vec<(String, PathBuf)> = if link.worktrees.is_empty() {
        link.dir
            .iter()
            .map(|d| (String::new(), PathBuf::from(d)))
            .collect()
    } else {
        link.worktrees
            .iter()
            .map(|w| (crate::taskspace::repo_name(&w.repo), PathBuf::from(&w.path)))
            .collect()
    };
    let mut parts = Vec::new();
    for (name, dir) in worktrees {
        let (commits, stat) = git_facts(&dir);
        let text = code_text(&commits, &stat);
        if text.is_empty() {
            continue;
        }
        parts.push(if name.is_empty() {
            text
        } else {
            format!("{name}:\n{text}")
        });
    }
    parts.join("\n\n")
}

/// The base a branch starts from: the remote default branch, or the last few commits when there is none.
fn git_facts(dir: &PathBuf) -> (String, String) {
    let has_origin = git(dir, &["rev-parse", "--verify", "--quiet", "origin/HEAD"]).is_some();
    let commits = if has_origin {
        git(
            dir,
            &["log", "--oneline", "--max-count=10", "origin/HEAD..HEAD"],
        )
    } else {
        git(dir, &["log", "--oneline", "--max-count=10"])
    }
    .unwrap_or_default();
    let stat = if has_origin {
        git(dir, &["diff", "--shortstat", "origin/HEAD"])
    } else {
        git(dir, &["diff", "--shortstat", "HEAD~5"]).or_else(|| git(dir, &["diff", "--shortstat"]))
    }
    .unwrap_or_default();
    let stat = if stat.trim().is_empty() {
        let status = git(dir, &["status", "--porcelain"]).unwrap_or_default();
        match status.lines().filter(|l| !l.trim().is_empty()).count() {
            0 => String::new(),
            1 => "1 changed file".to_string(),
            n => format!("{n} changed files"),
        }
    } else {
        stat
    };
    (commits, stat)
}

fn git(dir: &PathBuf, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).to_string())
}

/// The draft's code body: the commits, then the short diff stat.
pub fn code_text(commits: &str, stat: &str) -> String {
    let mut out = String::new();
    let lines: Vec<&str> = commits
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(MAX_COMMITS)
        .collect();
    if !lines.is_empty() {
        out.push_str(&format!(
            "{} commit{}:\n",
            lines.len(),
            if lines.len() == 1 { "" } else { "s" }
        ));
        for line in lines {
            out.push_str(&format!("- {line}\n"));
        }
    }
    let stat = stat.trim();
    if !stat.is_empty() {
        out.push_str(stat);
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// The draft's spec body: the proposal's Why, then the task count.
pub fn spec_text(why: &str, done: usize, total: usize) -> String {
    let mut out = String::new();
    if !why.trim().is_empty() {
        out.push_str(&format!("Why: {}", why.trim()));
    }
    if total > 0 {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&format!("Tasks: {done} of {total} done"));
    }
    out
}

/// The `## Why` section of a proposal, as one line, capped for a comment.
pub fn why_section(text: &str) -> String {
    let mut lines = Vec::new();
    let mut in_why = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("## why") {
            in_why = true;
            continue;
        }
        if in_why && trimmed.starts_with('#') {
            break;
        }
        if in_why && !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }
    cap(&lines.join(" "), MAX_WHY)
}

fn cap(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push('…');
    out
}

/// `(done, total)` checkboxes in a tasks file.
pub fn task_counts(text: &str) -> (usize, usize) {
    let mut done = 0;
    let mut total = 0;
    for line in text.lines().map(str::trim) {
        if line.starts_with("- [x]") || line.starts_with("- [X]") {
            done += 1;
            total += 1;
        } else if line.starts_with("- [ ]") {
            total += 1;
        }
    }
    (done, total)
}
