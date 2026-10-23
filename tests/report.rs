use herdr_clickup::hook::{Link, Phase, RepoWorktree};
use herdr_clickup::report::{code_text, spec_text, task_counts, why_section, work_summary};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "herdr-clickup-report-{name}-{}",
        std::process::id()
    ));
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

fn link(dir: &Path, change: Option<&str>) -> Link {
    Link {
        task_id: "1".into(),
        branch: "CU-1-fix".into(),
        change: change.map(str::to_string),
        phase: Some(Phase::SpecDrafted),
        dir: Some(dir.display().to_string()),
        worktrees: vec![RepoWorktree {
            repo: dir.display().to_string(),
            path: dir.display().to_string(),
        }],
        title: "Fix".into(),
        url: "u".into(),
        agent: "claude".into(),
        working: false,
        reported: true,
        pending: None,
    }
}

#[test]
fn the_why_section_is_one_capped_line() {
    let text =
        "# Proposal\n\n## Why\n\nThe\nPO button is missing.\n\n## What Changes\n\n- Add it\n";
    assert_eq!(why_section(text), "The PO button is missing.");
    assert_eq!(why_section("no why"), "");
    let long = format!("## Why\n\n{}", "x".repeat(600));
    assert!(why_section(&long).chars().count() <= 501);
    assert!(why_section(&long).ends_with('…'));
}

#[test]
fn task_boxes_are_counted() {
    let text = "# Tasks\n\n- [x] one\n- [ ] two\n- [X] three\n* [ ] not counted\n";
    assert_eq!(task_counts(text), (2, 3));
    assert_eq!(task_counts(""), (0, 0));
}

#[test]
fn a_code_draft_has_commits_then_the_file_stat() {
    let text = code_text(
        "abc fix login\ndef add test\n",
        "3 files changed, 42 insertions(+), 7 deletions(-)",
    );
    assert_eq!(
        text,
        "2 commits:\n- abc fix login\n- def add test\n3 files changed, 42 insertions(+), 7 deletions(-)"
    );
    assert_eq!(code_text("", "1 changed file"), "1 changed file");
    assert_eq!(code_text("", ""), "");
    assert_eq!(code_text("one", ""), "1 commit:\n- one");
}

#[test]
fn a_spec_draft_has_the_why_and_the_task_count() {
    assert_eq!(
        spec_text("The button is missing.", 2, 5),
        "Why: The button is missing.\n\nTasks: 2 of 5 done"
    );
    assert_eq!(spec_text("", 0, 0), "");
}

#[test]
fn a_code_summary_reads_the_worktree() {
    let dir = temp("code");
    run(&dir, &["init", "-q", "-b", "main"]);
    run(&dir, &["config", "user.email", "test@example.com"]);
    run(&dir, &["config", "user.name", "Test"]);
    fs::write(dir.join("README.md"), "x\n").unwrap();
    run(&dir, &["add", "."]);
    run(&dir, &["commit", "-qm", "init"]);
    fs::write(dir.join("README.md"), "y\n").unwrap();
    run(&dir, &["commit", "-qam", "add the button"]);
    fs::write(dir.join("new.txt"), "z\n").unwrap();
    let summary = work_summary(&link(&dir, None));
    assert!(summary.contains("add the button"), "{summary}");
    assert!(summary.contains("changed file"), "{summary}");
}

#[test]
fn a_spec_summary_reads_the_proposal_and_tasks() {
    let dir = temp("spec");
    let change = dir.join("openspec/changes/cu-1-fix");
    fs::create_dir_all(&change).unwrap();
    fs::write(
        change.join("proposal.md"),
        "# Proposal\n\n## Why\n\nThe PO button is missing.\n",
    )
    .unwrap();
    fs::write(change.join("tasks.md"), "- [x] one\n- [ ] two\n").unwrap();
    let summary = work_summary(&link(&dir, Some("cu-1-fix")));
    assert_eq!(
        summary,
        "Why: The PO button is missing.\n\nTasks: 1 of 2 done"
    );
}
