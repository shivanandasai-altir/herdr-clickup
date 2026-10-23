use herdr_clickup::hook::{decide, Decision, Link, State};
use serde_json::json;

fn state(reported: bool) -> State {
    let mut s = State::default();
    s.panes.insert(
        "w3:t1:p1".into(),
        Link { task_id: "86c1abcd".into(), branch: "CU-86c1abcd-fix-login".into(), reported },
    );
    s
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
fn other_statuses_do_nothing() {
    for s in ["working", "idle", "blocked", "unknown"] {
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
