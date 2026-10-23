use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::{env, process::Command};

/// Calls the running herdr through its CLI (`$HERDR_BIN_PATH`), which prints JSON.
pub fn call(args: &[&str]) -> Result<Value> {
    let bin = env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into());
    let out = Command::new(&bin).args(args).output().with_context(|| format!("running {bin}"))?;
    if !out.status.success() {
        bail!("herdr {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    Ok(v.get("result").cloned().unwrap_or(v))
}

/// The workspace the plugin was invoked from: the env var, else the context JSON.
pub fn current_workspace() -> Option<String> {
    if let Ok(w) = env::var("HERDR_WORKSPACE_ID") {
        return Some(w);
    }
    let ctx: Value = serde_json::from_str(&env::var("HERDR_PLUGIN_CONTEXT_JSON").ok()?).ok()?;
    ["/workspace_id", "/workspace/workspace_id", "/workspace/id"]
        .iter()
        .find_map(|p| ctx.pointer(p).and_then(Value::as_str).map(str::to_string))
}

pub struct Worktree {
    pub workspace_id: String,
    pub pane_id: String,
}

/// Creates a worktree on `branch` from the current workspace's repo and opens it.
pub fn create_worktree(branch: &str, label: &str) -> Result<Worktree> {
    let mut args = vec!["worktree", "create", "--branch", branch, "--label", label, "--focus"];
    let ws = current_workspace();
    if let Some(w) = ws.as_deref() {
        args.extend(["--workspace", w]);
    }
    let r = call(&args)?;
    let field = |p: &str| r.pointer(p).and_then(Value::as_str).map(str::to_string);
    Ok(Worktree {
        workspace_id: field("/workspace/workspace_id").ok_or_else(|| anyhow!("worktree create: no workspace in {r}"))?,
        pane_id: field("/root_pane/pane_id").ok_or_else(|| anyhow!("worktree create: no root pane in {r}"))?,
    })
}

pub fn start_agent(name: &str, kind: &str, pane: &str) -> Result<()> {
    call(&["agent", "start", name, "--kind", kind, "--pane", pane]).map(drop)
}

pub fn prompt_agent(pane: &str, text: &str) -> Result<()> {
    call(&["agent", "prompt", pane, text]).map(drop)
}

pub fn notify(title: &str, body: &str) {
    let _ = call(&["notification", "show", title, "--body", body, "--sound", "done"]);
}

/// Opens one of this plugin's panes, passing `env` (`KEY=VALUE`) to its process.
pub fn open_pane(entrypoint: &str, env: &[String]) -> Result<()> {
    let id = env::var("HERDR_PLUGIN_ID").unwrap_or_else(|_| "ssk090.clickup".into());
    let mut args = vec!["plugin", "pane", "open", "--plugin", &id, "--entrypoint", entrypoint];
    for e in env {
        args.extend(["--env", e]);
    }
    call(&args).map(drop)
}
