use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{env, process::Command};

/// herdr's answer to one CLI call: the `result` on success, else a readable error. herdr reports
/// request errors as `{"error":{"code","message"}}` on stdout, sometimes with exit status 0.
pub fn parse_output(success: bool, stdout: &str, stderr: &str) -> std::result::Result<Value, String> {
    let v: Value = serde_json::from_str(stdout).unwrap_or(Value::Null);
    let err_json: Value = serde_json::from_str(stderr.trim()).unwrap_or(Value::Null);
    if let Some(e) = v.get("error").or_else(|| err_json.get("error")) {
        let text = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
        return Err(match (text("message"), text("code")) {
            (m, c) if c.is_empty() => m,
            (m, c) if m.is_empty() => c,
            (m, c) => format!("{m} ({c})"),
        });
    }
    if !success {
        return Err([stderr.trim(), stdout.trim()]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or("failed with no output")
            .to_string());
    }
    Ok(v.get("result").cloned().unwrap_or(v))
}

/// Calls the running herdr through its CLI (`$HERDR_BIN_PATH`), which prints JSON.
pub fn call(args: &[&str]) -> Result<Value> {
    let bin = env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into());
    let out = Command::new(&bin).args(args).output().with_context(|| format!("running {bin}"))?;
    let (stdout, stderr) = (String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    // The reason first: a TUI status line only has room for the start of the message.
    parse_output(out.status.success(), &stdout, &stderr).map_err(|e| anyhow!("{e} — herdr {}", args.join(" ")))
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
    /// The checkout's folder.
    pub path: String,
}

/// Creates a worktree of `repo` on `branch` and opens it in a new workspace.
pub fn create_worktree(repo: &str, branch: &str, label: &str) -> Result<Worktree> {
    let r = call(&["worktree", "create", "--cwd", repo, "--branch", branch, "--label", label, "--focus"])?;
    let field = |p: &str| r.pointer(p).and_then(Value::as_str).map(str::to_string);
    Ok(Worktree {
        workspace_id: field("/workspace/workspace_id").ok_or_else(|| anyhow!("worktree create: no workspace in {r}"))?,
        pane_id: field("/root_pane/pane_id").ok_or_else(|| anyhow!("worktree create: no root pane in {r}"))?,
        path: field("/worktree/path").ok_or_else(|| anyhow!("worktree create: no path in {r}"))?,
    })
}

pub fn start_agent(name: &str, kind: &str, pane: &str) -> Result<()> {
    call(&["agent", "start", name, "--kind", kind, "--pane", pane]).map(drop)
}

/// Waits until the agent in `pane` is ready for a prompt (idle), up to `timeout_ms`.
pub fn wait_until_idle(pane: &str, timeout_ms: u64) -> Result<()> {
    call(&["agent", "wait", pane, "--until", "idle", "--timeout", &timeout_ms.to_string()]).map(drop)
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
