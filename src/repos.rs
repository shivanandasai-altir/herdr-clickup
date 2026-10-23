//! Which git repo a task's worktree goes in.
use crate::herdr;
use serde_json::Value;
use std::{env, process::Command};

/// The repos to offer: the current workspace's, then the last pick, then the rest in workspace order.
pub fn repo_choices(found: &[String], current: Option<&str>, last: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in current.into_iter().chain(last).chain(found.iter().map(String::as_str)) {
        if !r.is_empty() && !out.iter().any(|o| o == r) {
            out.push(r.to_string());
        }
    }
    out
}

/// The git top-level of `dir`, if it is inside a work tree.
fn toplevel(dir: &str) -> Option<String> {
    let out = Command::new("git").args(["-C", dir, "rev-parse", "--show-toplevel"]).output().ok()?;
    let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !top.is_empty()).then_some(top)
}

/// `(workspace id, repo)` for every herdr pane sitting in a git repo, skipping this plugin's own pane
/// (it runs from the plugin's checkout, which is not where tasks belong).
pub fn open_repos() -> Vec<(String, String)> {
    let me = env::var("HERDR_PANE_ID").ok();
    let Ok(v) = herdr::call(&["pane", "list"]) else { return Vec::new() };
    let panes = v.get("panes").and_then(Value::as_array).cloned().unwrap_or_default();
    panes
        .iter()
        .filter(|p| p.get("pane_id").and_then(Value::as_str) != me.as_deref())
        .filter_map(|p| {
            let ws = p.get("workspace_id").and_then(Value::as_str).map(str::to_string).or_else(|| {
                // Pane ids are `<workspace>:<pane>`.
                p.get("pane_id")?.as_str()?.split(':').next().map(str::to_string)
            })?;
            let cwd = ["foreground_cwd", "cwd"].iter().find_map(|k| p.get(*k).and_then(Value::as_str))?;
            Some((ws, toplevel(cwd)?))
        })
        .collect()
}

/// The repos to offer for a start from the current herdr workspace.
pub fn choices(last: Option<&str>) -> Vec<String> {
    let open = open_repos();
    let ws = herdr::current_workspace();
    let current = open.iter().find(|(w, _)| Some(w) == ws.as_ref()).map(|(_, r)| r.as_str());
    let found: Vec<String> = open.iter().map(|(_, r)| r.clone()).collect();
    repo_choices(&found, current, last)
}
