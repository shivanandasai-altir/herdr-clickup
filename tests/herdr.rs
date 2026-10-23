use herdr_clickup::herdr::parse_output;
use serde_json::json;

#[test]
fn success_returns_the_result() {
    let out = r#"{"id":"cli:x","result":{"type":"worktree_created","root_pane":{"pane_id":"w3:p1"}}}"#;
    assert_eq!(parse_output(true, out, ""), Ok(json!({ "type": "worktree_created", "root_pane": { "pane_id": "w3:p1" } })));
}

#[test]
fn an_error_object_fails_even_with_exit_status_zero() {
    let out = r#"{"error":{"code":"not_git_worktree","message":"Herdr worktree actions require a workspace inside a Git work tree"},"id":"cli:worktree:create"}"#;
    assert_eq!(
        parse_output(true, out, ""),
        Err("Herdr worktree actions require a workspace inside a Git work tree (not_git_worktree)".into())
    );
}

#[test]
fn a_failed_exit_uses_the_error_object_before_stderr() {
    let out = r#"{"error":{"code":"agent_not_ready","message":"agent did not become ready"}}"#;
    assert_eq!(parse_output(false, out, "noise"), Err("agent did not become ready (agent_not_ready)".into()));
}

#[test]
fn a_failed_exit_without_json_uses_stderr_then_stdout() {
    assert_eq!(parse_output(false, "", "  error: unexpected argument '--foo'\n"), Err("error: unexpected argument '--foo'".into()));
    assert_eq!(parse_output(false, "plain text failure\n", ""), Err("plain text failure".into()));
    assert_eq!(parse_output(false, "", ""), Err("failed with no output".into()));
}

#[test]
fn success_without_json_is_null() {
    assert_eq!(parse_output(true, "", ""), Ok(serde_json::Value::Null));
}

#[test]
fn an_error_object_on_stderr_is_read_too() {
    let err = r#"{"error":{"code":"worktree_create_failed","message":"fatal: '/w/x' already exists"},"id":"cli:worktree:create"}"#;
    assert_eq!(parse_output(false, "", err), Err("fatal: '/w/x' already exists (worktree_create_failed)".into()));
}
