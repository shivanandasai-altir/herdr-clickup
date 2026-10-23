use herdr_clickup::hook::{
    Decision, Link, Phase, RepoWorktree, State, Update, decide, proposed_update,
};
use serde_json::json;

fn link(working: bool, reported: bool) -> State {
    let mut s = State::default();
    s.panes.insert(
        "w3:t1:p1".into(),
        Link {
            task_id: "86c1abcd".into(),
            branch: "CU-86c1abcd-fix-login".into(),
            change: None,
            phase: None,
            dir: None,
            worktrees: Vec::new(),
            title: String::new(),
            url: String::new(),
            agent: String::new(),
            working,
            reported,
            pending: None,
        },
    );
    s
}

// A pane whose agent has started on the task and not been reported yet.
fn state(reported: bool) -> State {
    link(true, reported)
}

// Captured from herdr 0.9.5's HERDR_PLUGIN_EVENT_JSON: event names use underscores, though the
// manifest subscribes with dots (`pane.agent_status_changed`).
fn status_event(pane: &str, status: &str) -> serde_json::Value {
    json!({
        "event": "pane_agent_status_changed",
        "data": { "type": "pane_agent_status_changed", "pane_id": pane, "workspace_id": "w3", "agent_status": status, "agent": "claude" }
    })
}

#[test]
fn dotted_event_names_are_accepted_too() {
    let ev = json!({ "event": "pane.agent_status_changed", "data": { "pane_id": "w3:t1:p1", "agent_status": "done" } });
    assert!(matches!(
        decide(&ev, &state(false)),
        Decision::Report { .. }
    ));
}

#[test]
fn done_on_a_linked_pane_reports_the_task() {
    assert_eq!(
        decide(&status_event("w3:t1:p1", "done"), &state(false)),
        Decision::Report {
            pane_id: "w3:t1:p1".into(),
            task_id: "86c1abcd".into(),
            branch: "CU-86c1abcd-fix-login".into(),
            change: None,
            phase: Phase::Implementing,
        }
    );
}

#[test]
fn reports_only_once() {
    assert_eq!(
        decide(&status_event("w3:t1:p1", "done"), &state(true)),
        Decision::Nothing
    );
}

#[test]
fn idle_after_working_also_reports() {
    // herdr says idle, not done, when the finish was already seen (the pane was focused).
    assert!(matches!(
        decide(&status_event("w3:t1:p1", "idle"), &state(false)),
        Decision::Report { .. }
    ));
}

#[test]
fn idle_or_done_before_the_agent_worked_does_nothing() {
    // The agent sits idle between `agent start` and the first prompt.
    for s in ["idle", "done"] {
        assert_eq!(
            decide(&status_event("w3:t1:p1", s), &link(false, false)),
            Decision::Nothing,
            "{s}"
        );
    }
}

#[test]
fn first_working_marks_the_link_working() {
    assert_eq!(
        decide(&status_event("w3:t1:p1", "working"), &link(false, false)),
        Decision::MarkWorking {
            pane_id: "w3:t1:p1".into()
        }
    );
}

#[test]
fn other_statuses_do_nothing() {
    for s in ["working", "blocked", "unknown"] {
        assert_eq!(
            decide(&status_event("w3:t1:p1", s), &state(false)),
            Decision::Nothing,
            "{s}"
        );
    }
}

#[test]
fn unlinked_panes_do_nothing() {
    assert_eq!(
        decide(&status_event("w9:t1:p1", "done"), &state(false)),
        Decision::Nothing
    );
}

#[test]
fn closing_a_linked_pane_forgets_it() {
    let ev =
        json!({ "event": "pane_closed", "data": { "type": "pane_closed", "pane_id": "w3:t1:p1" } });
    assert_eq!(
        decide(&ev, &state(true)),
        Decision::Forget {
            pane_id: "w3:t1:p1".into()
        }
    );
}

#[test]
fn garbage_does_nothing() {
    assert_eq!(decide(&json!(null), &state(false)), Decision::Nothing);
    assert_eq!(
        decide(&json!({ "event": "pane.closed" }), &state(false)),
        Decision::Nothing
    );
}

#[test]
fn removing_a_worktree_forgets_its_workspaces_links() {
    let ev = json!({ "event": "worktree_removed", "data": { "type": "worktree_removed", "workspace_id": "w3", "forced": true, "worktree": {} } });
    assert_eq!(
        decide(&ev, &state(false)),
        Decision::ForgetWorkspace {
            workspace_id: "w3".into()
        }
    );
}

#[test]
fn removing_an_unlinked_worktree_does_nothing() {
    // w30 must not match the w3:… panes.
    let ev = json!({ "event": "worktree_removed", "data": { "workspace_id": "w30" } });
    assert_eq!(decide(&ev, &state(false)), Decision::Nothing);
}

#[test]
fn a_spec_start_reports_with_its_change() {
    let mut s = state(false);
    s.panes.get_mut("w3:t1:p1").unwrap().change = Some("cu-86c1abcd-fix-login".into());
    assert_eq!(
        decide(&status_event("w3:t1:p1", "idle"), &s),
        Decision::Report {
            pane_id: "w3:t1:p1".into(),
            task_id: "86c1abcd".into(),
            branch: "CU-86c1abcd-fix-login".into(),
            change: Some("cu-86c1abcd-fix-login".into()),
            phase: Phase::SpecDrafting,
        }
    );
}

#[test]
fn a_finished_spec_start_proposes_a_spec_drafted_update() {
    assert_eq!(
        proposed_update(
            "CU-86a-x",
            Some("cu-86a-x"),
            Phase::SpecDrafting,
            Some("review"),
            None
        ),
        Update {
            comment:
                "Spec drafted: OpenSpec change `cu-86a-x` on branch `CU-86a-x` is ready for review."
                    .into(),
            status: None
        }
    );
}

#[test]
fn a_finished_code_start_proposes_finished_and_the_done_status() {
    assert_eq!(
        proposed_update(
            "CU-86a-x",
            None,
            Phase::Implementing,
            Some("review"),
            Some("spec review")
        ),
        Update {
            comment: "herdr agent finished its turn on branch `CU-86a-x`.".into(),
            status: Some("review".into())
        }
    );
}

#[test]
fn a_link_written_before_phases_existed_reads_its_phase_from_the_change() {
    let spec: Link =
        serde_json::from_value(json!({ "task_id": "1", "branch": "b", "change": "c" })).unwrap();
    assert_eq!(spec.phase(), Phase::SpecDrafting);
    let code: Link = serde_json::from_value(json!({ "task_id": "1", "branch": "b" })).unwrap();
    assert_eq!(code.phase(), Phase::Implementing);
    let done: Link =
        serde_json::from_value(json!({ "task_id": "1", "branch": "b", "reported": true })).unwrap();
    assert_eq!(done.phase(), Phase::Done);
    let finished: Link = serde_json::from_value(
        json!({ "task_id": "1", "branch": "b", "change": "c", "reported": true }),
    )
    .unwrap();
    assert_eq!(finished.phase(), Phase::SpecDrafted);
}

#[test]
fn a_finished_implementation_proposes_the_done_status() {
    assert_eq!(
        proposed_update("CU-86a-x", Some("cu-86a-x"), Phase::Implementing, Some("review"), Some("spec review")),
        Update {
            comment: "Implementation done: OpenSpec change `cu-86a-x` on branch `CU-86a-x` is ready for review.".into(),
            status: Some("review".into()),
        }
    );
}

#[test]
fn implementing_claims_the_next_finish_again() {
    let mut l = Link {
        task_id: "1".into(),
        branch: "b".into(),
        change: Some("c".into()),
        phase: Some(Phase::SpecDrafted),
        dir: None,
        worktrees: Vec::new(),
        title: String::new(),
        url: String::new(),
        agent: String::new(),
        working: true,
        reported: true,
        pending: Some(Update {
            comment: "spec drafted".into(),
            status: None,
        }),
    };
    l.implementing();
    assert_eq!(
        (l.phase(), l.working, l.reported),
        (Phase::Implementing, false, false)
    );
}

#[test]
fn a_report_advances_the_phase() {
    let mut l = Link {
        task_id: "1".into(),
        branch: "b".into(),
        change: Some("c".into()),
        phase: Some(Phase::SpecDrafting),
        dir: None,
        worktrees: Vec::new(),
        title: String::new(),
        url: String::new(),
        agent: String::new(),
        working: true,
        reported: false,
        pending: None,
    };
    l.reported_as();
    assert_eq!(l.phase(), Phase::SpecDrafted);
    l.implementing();
    l.reported_as();
    assert_eq!(l.phase(), Phase::Done);
}

#[test]
fn phases_have_short_board_labels() {
    assert_eq!(
        [
            Phase::SpecDrafting,
            Phase::SpecDrafted,
            Phase::Implementing,
            Phase::Done
        ]
        .map(Phase::label),
        ["spec…", "spec ✓", "code…", "done"]
    );
}

#[test]
fn implement_needs_a_reviewed_change() {
    let mut l = Link {
        task_id: "1".into(),
        branch: "b".into(),
        change: Some("c".into()),
        phase: Some(Phase::SpecDrafted),
        dir: None,
        worktrees: Vec::new(),
        title: String::new(),
        url: String::new(),
        agent: String::new(),
        working: false,
        reported: true,
        pending: None,
    };
    assert!(l.can_implement());
    l.working = true;
    assert!(l.can_implement(), "the stale working flag does not decide");
    l.working = false;
    l.phase = Some(Phase::SpecDrafting);
    assert!(!l.can_implement());
    l.phase = Some(Phase::Implementing);
    assert!(!l.can_implement());
    l.change = None;
    l.phase = Some(Phase::SpecDrafted);
    assert!(!l.can_implement());
}

fn task_link(dir: &str, paths: &[&str]) -> Link {
    Link {
        task_id: "1".into(),
        branch: "CU-1-fix".into(),
        change: Some("cu-1-fix".into()),
        phase: Some(Phase::SpecDrafted),
        dir: Some(dir.into()),
        worktrees: paths
            .iter()
            .map(|p| RepoWorktree {
                repo: format!("{p}-repo"),
                path: p.to_string(),
            })
            .collect(),
        title: "Fix".into(),
        url: "u".into(),
        agent: "claude".into(),
        working: false,
        reported: true,
        pending: None,
    }
}

#[test]
fn a_single_worktree_is_not_a_task_folder() {
    assert_eq!(task_link("/t/CU-1-fix", &["/t/CU-1-fix"]).folder(), None);
}

#[test]
fn a_worktree_inside_the_dir_makes_it_a_task_folder() {
    assert_eq!(
        task_link("/t/CU-1-fix", &["/t/CU-1-fix/webui"]).folder(),
        Some("/t/CU-1-fix")
    );
}

#[test]
fn links_with_repos_survive_a_json_round_trip() {
    let l = task_link("/t/CU-1-fix", &["/t/CU-1-fix/webui", "/t/CU-1-fix/api"]);
    let text = serde_json::to_string(&l).unwrap();
    let back: Link = serde_json::from_str(&text).unwrap();
    assert_eq!(back, l);
    assert_eq!(back.worktrees.len(), 2);
}
