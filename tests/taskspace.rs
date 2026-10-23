use herdr_clickup::hook::RepoWorktree;
use herdr_clickup::taskspace::{
    add_repo, add_worktrees, base_ref, dirty, dirty_dir, read_repos, remove_worktrees, repo_name,
    same_repo, task_dir, unique_name, write_agents_md, write_repos,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("herdr-clickup-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn run(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn repo(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    run(dir, &["init", "-q", "-b", "main"]);
    run(dir, &["config", "user.email", "test@example.com"]);
    run(dir, &["config", "user.name", "Test"]);
    fs::write(dir.join("README.md"), "hello\n").unwrap();
    run(dir, &["add", "."]);
    run(dir, &["commit", "-qm", "init"]);
}

#[test]
fn task_folders_live_under_the_herdr_home() {
    let dir = task_dir("CU-1-demo").unwrap();
    assert!(dir.ends_with(".herdr/tasks/CU-1-demo"), "{}", dir.display());
}

#[test]
fn repo_names_come_from_the_last_path_segment() {
    assert_eq!(repo_name("/w/chip1-webui/"), "chip1-webui");
    assert_eq!(repo_name("chip1-api-hub"), "chip1-api-hub");
    assert_eq!(repo_name("/"), "repo");
}

#[test]
fn a_duplicate_repo_name_gets_a_suffix() {
    let taken = [RepoWorktree {
        repo: "/a/webui".into(),
        path: "/t/webui".into(),
    }];
    assert_eq!(unique_name("/b/webui", &taken), "webui-2");
    assert_eq!(unique_name("/b/other", &taken), "other");
}

#[test]
fn a_repo_list_skips_comments_blanks_and_duplicates() {
    let dir = temp("repos");
    write_repos(&dir, &["/w/a".into(), "/w/b".into()]).unwrap();
    fs::write(dir.join("repos.txt"), "# comment\n\n/w/a\n/w/b\n/w/a\n").unwrap();
    assert_eq!(read_repos(&dir).unwrap(), ["/w/a", "/w/b"]);
    assert_eq!(read_repos(&temp("missing")).unwrap(), Vec::<String>::new());
}

#[test]
fn a_relative_repo_path_is_rejected() {
    let dir = temp("relative");
    fs::write(dir.join("repos.txt"), "webui\n").unwrap();
    assert!(read_repos(&dir).is_err());
}

#[test]
fn adding_a_repo_keeps_an_existing_list() {
    let dir = temp("add-repo");
    write_repos(&dir, &["/w/a".into()]).unwrap();
    add_repo(&dir, "/w/b").unwrap();
    add_repo(&dir, "/w/a").unwrap();
    assert_eq!(read_repos(&dir).unwrap(), ["/w/a", "/w/b"]);
}

#[test]
fn the_task_instructions_name_the_branch_and_every_repo() {
    let dir = temp("instructions");
    let worktrees = vec![
        RepoWorktree {
            repo: "/w/webui".into(),
            path: dir.join("webui").display().to_string(),
        },
        RepoWorktree {
            repo: "/w/api".into(),
            path: dir.join("api").display().to_string(),
        },
    ];
    write_agents_md(
        &dir,
        "Fix the PO button",
        "https://app.clickup.com/t/1",
        "CU-1-fix",
        &worktrees,
        "codex",
    )
    .unwrap();
    let md = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        md.contains("CU-1-fix") && md.contains("webui") && md.contains("api"),
        "{md}"
    );
    assert!(!dir.join("CLAUDE.md").exists());
    write_agents_md(&dir, "Fix", "u", "CU-1", &worktrees, "claude").unwrap();
    assert!(dir.join("CLAUDE.md").exists());
}

#[test]
fn worktrees_are_created_detected_dirty_and_removed() {
    let root = temp("worktrees");
    let repo_a = root.join("webui");
    let repo_b = root.join("api");
    repo(&repo_a);
    repo(&repo_b);
    assert_eq!(base_ref(repo_a.to_str().unwrap()), "HEAD");
    let task = root.join("task");
    let added = add_worktrees(
        &task,
        &[repo_a.display().to_string(), repo_b.display().to_string()],
        "CU-1-fix",
        &[],
    )
    .unwrap();
    assert_eq!(added.len(), 2);
    assert!(
        added
            .iter()
            .all(|w| Path::new(&w.path).join("README.md").exists())
    );
    let branch = Command::new("git")
        .args(["-C", &added[0].path, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), "CU-1-fix");
    assert!(dirty(&added).is_empty());
    fs::write(Path::new(&added[0].path).join("new.txt"), "x").unwrap();
    assert_eq!(dirty(&added), [repo_a.display().to_string()]);
    let again = add_worktrees(&task, &[repo_a.display().to_string()], "CU-1-fix", &added).unwrap();
    assert!(again.is_empty(), "a repo already in the task is skipped");
    assert!(remove_worktrees(&added).is_empty());
    assert!(added.iter().all(|w| !Path::new(&w.path).exists()));
    assert!(
        remove_worktrees(&added).is_empty(),
        "a retry after a partial close is harmless"
    );
}

#[test]
fn two_repos_with_the_same_name_get_distinct_folders() {
    let root = temp("same-name");
    let a = root.join("one").join("webui");
    let b = root.join("two").join("webui");
    repo(&a);
    repo(&b);
    let added = add_worktrees(
        &root.join("task"),
        &[a.display().to_string(), b.display().to_string()],
        "CU-1-fix",
        &[],
    )
    .unwrap();
    let names: Vec<String> = added
        .iter()
        .map(|w| {
            Path::new(&w.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(names, ["webui", "webui-2"]);
    assert!(remove_worktrees(&added).is_empty());
}

#[test]
fn a_worktree_starts_from_the_remote_default_branch() {
    let root = temp("origin-head");
    let bare = root.join("remote.git");
    let out = Command::new("git")
        .args(["init", "--bare", "-q", "--initial-branch=main"])
        .arg(&bare)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let repo_dir = root.join("webui");
    let out = Command::new("git")
        .args(["clone", "-q"])
        .arg(&bare)
        .arg(&repo_dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    run(&repo_dir, &["config", "user.email", "test@example.com"]);
    run(&repo_dir, &["config", "user.name", "Test"]);
    fs::write(repo_dir.join("README.md"), "remote\n").unwrap();
    run(&repo_dir, &["add", "."]);
    run(&repo_dir, &["commit", "-qm", "init"]);
    run(&repo_dir, &["push", "-qu", "origin", "main"]);
    run(&repo_dir, &["remote", "set-head", "origin", "main"]);
    assert_eq!(base_ref(repo_dir.to_str().unwrap()), "origin/HEAD");
    let added = add_worktrees(
        &root.join("task"),
        &[repo_dir.display().to_string()],
        "CU-2-fix",
        &[],
    )
    .unwrap();
    let head = Command::new("git")
        .args(["-C", &added[0].path, "rev-parse", "HEAD"])
        .output()
        .unwrap();
    let remote = Command::new("git")
        .args(["-C", &added[0].path, "rev-parse", "origin/main"])
        .output()
        .unwrap();
    assert_eq!(head.stdout, remote.stdout);
    assert!(remove_worktrees(&added).is_empty());
}

#[test]
fn dirty_dir_and_same_repo_see_a_real_checkout() {
    let root = temp("dirty-dir");
    let r = root.join("repo");
    repo(&r);
    let path = r.display().to_string();
    assert!(!dirty_dir(&path));
    fs::write(r.join("new.txt"), "x").unwrap();
    assert!(dirty_dir(&path));
    assert!(same_repo(&format!("{path}/"), &path));
}
