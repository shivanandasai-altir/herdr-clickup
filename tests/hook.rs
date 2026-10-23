use herdr_clickup::hook::{decide, Decision, Link, State};
use serde_json::json;

fn link(working: bool, reported: bool) -> State {
    let mut s = State::default();
    s.panes.insert(
        "w3:t1:p1".into(),
        Link { task_id: "86c1abcd".into(), branch: "CU-86c1abcd-fix-login".into(), working, reported },
    );
    s
}

// A pane whose agent has started on the task and not been reported yet.
fn state(reported: bool) -> State {
    link(true, reported)
}

// The shape herdr 0.9.x puts in HERDR_PLUGIN_EVENT_JSON.
fn status_event(pane: &str, status: &str) -> serde_json::Value {
    json!({
        "event": "pane.agent_status_changed",
        "data": { "pane_id": pane, "workspace_id": "w3", "agent_status": status, "agent": "claude" }
    })
}

#[test]
fn done_on_a_linked_pane_reports_the_task() {
    assert_eq!(
        decide(&status_event("w3:t1:p1", "done"), &state(false)),
        Decision::Report {
            pane_id: "w3:t1:p1".into(),
            task_id: "86c1abcd".into(),
            branch: "CU-86c1abcd-fix-login".into(),
        }
    );
}

#[test]
fn reports_only_once() {
    assert_eq!(decide(&status_event("w3:t1:p1", "done"), &state(true)), Decision::Nothing);
}

#[test]
fn idle_after_working_also_reports() {
    // herdr says idle, not done, when the finish was already seen (the pane was focused).
    assert!(matches!(decide(&status_event("w3:t1:p1", "idle"), &state(false)), Decision::Report { .. }));
}

#[test]
fn idle_or_done_before_the_agent_worked_does_nothing() {
    // The agent sits idle between `agent start` and the first prompt.
    for s in ["idle", "done"] {
        assert_eq!(decide(&status_event("w3:t1:p1", s), &link(false, false)), Decision::Nothing, "{s}");
    }
}

#[test]
fn first_working_marks_the_link_working() {
    assert_eq!(
        decide(&status_event("w3:t1:p1", "working"), &link(false, false)),
        Decision::MarkWorking { pane_id: "w3:t1:p1".into() }
    );
}

#[test]
fn other_statuses_do_nothing() {
    for s in ["working", "blocked", "unknown"] {
        assert_eq!(decide(&status_event("w3:t1:p1", s), &state(false)), Decision::Nothing, "{s}");
    }
}

#[test]
fn unlinked_panes_do_nothing() {
    assert_eq!(decide(&status_event("w9:t1:p1", "done"), &state(false)), Decision::Nothing);
}

#[test]
fn closing_a_linked_pane_forgets_it() {
    let ev = json!({ "event": "pane.closed", "data": { "pane_id": "w3:t1:p1" } });
    assert_eq!(decide(&ev, &state(true)), Decision::Forget { pane_id: "w3:t1:p1".into() });
}

#[test]
fn garbage_does_nothing() {
    assert_eq!(decide(&json!(null), &state(false)), Decision::Nothing);
    assert_eq!(decide(&json!({ "event": "pane.closed" }), &state(false)), Decision::Nothing);
}
