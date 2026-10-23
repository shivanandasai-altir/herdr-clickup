//! What the picker, the board and the CLI share: setup, agent choices, and starting a task.
use crate::{
    agents::{agent_choices, is_installed},
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::Link,
    store,
    task::{agent_name, agent_prompt, branch_name, Task},
};
use anyhow::{bail, Result};

pub fn client() -> Result<(Config, ClickUp)> {
    let cfg = Config::load()?;
    let cu = ClickUp::new(cfg.token.clone(), cfg.team_id.clone());
    Ok((cfg, cu))
}

/// Installed agents, the last one picked (else `CLICKUP_AGENT`) first.
pub fn agent_options(cfg: &Config) -> Vec<String> {
    let preferred = store::last_agent().or_else(|| cfg.agent.clone());
    agent_choices(is_installed, preferred.as_deref())
}

/// Worktree on `CU-<id>-<slug>`, `agent` in it with the task as its prompt, and the task marked started.
/// Returns a one-line summary; prints nothing, so a TUI can call it.
pub fn start(cfg: &Config, cu: &ClickUp, task: &Task, agent: &str) -> Result<String> {
    if agent.is_empty() {
        bail!("no agent chosen");
    }
    store::remember_agent(agent);
    let branch = branch_name(task);
    let wt = herdr::create_worktree(&branch, &task.name)?;
    // Linked before the agent starts, so the hook sees its first status change.
    let link = Link { task_id: task.id.clone(), branch: branch.clone(), working: false, reported: false };
    store::update(|state| state.panes.insert(wt.pane_id.clone(), link))?;
    herdr::start_agent(&agent_name(task), agent, &wt.pane_id)?;
    herdr::prompt_agent(&wt.pane_id, &agent_prompt(task))?;

    let mut summary = format!("{agent} started on {branch}");
    if let Some(warning) = set_status(cu, &task.id, cfg.status_started.as_deref()) {
        summary.push_str(&format!(" ({warning})"));
    }
    herdr::notify("ClickUp task started", &format!("{} → {agent} in {}", task.name, wt.workspace_id));
    Ok(summary)
}

/// Status names differ per ClickUp space, so a missing one is a warning, not a failure.
pub fn set_status(cu: &ClickUp, task_id: &str, status: Option<&str>) -> Option<String> {
    let s = status?;
    cu.set_status(task_id, s).err().map(|e| format!("could not set status \"{s}\": {e:#}"))
}
