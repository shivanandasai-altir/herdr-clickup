//! What the picker, the board and the CLI share: setup, agent choices, and starting a task.
use crate::{
    agents::{agent_choices, is_installed},
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::Link,
    repos,
    store,
    openspec,
    task::{agent_name, branch_name, Task},
    ticket::{code_prompt, spec_prompt, ticket_markdown},
};
use anyhow::{bail, Result};
use std::{fs, path::Path, process::Command};

/// How long a start waits for the user to answer an agent's startup question.
const READY_TIMEOUT_MS: u64 = 15 * 60 * 1000;

pub const NO_AGENT: &str = "no supported coding agent is installed";
pub const NO_REPO: &str = "no git repo is open in a herdr workspace; open the task's repo in herdr first";

pub fn client() -> Result<(Config, ClickUp)> {
    let cfg = Config::load()?;
    let cu = ClickUp::new(cfg.token.clone(), cfg.team_id.clone());
    Ok((cfg, cu))
}

/// Installed agents, the last one picked (else `CLICKUP_AGENT`) first.
pub fn agent_options(cfg: &Config) -> Vec<String> {
    let preferred = store::last("agent").or_else(|| cfg.agent.clone());
    agent_choices(is_installed, preferred.as_deref())
}

/// Repos open in herdr workspaces, the current workspace's first, then the last one picked.
pub fn repo_options() -> Vec<String> {
    repos::choices(store::last("repo").as_deref())
}

/// How a start begins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Write an OpenSpec change (proposal, specs, design, tasks) from the ticket; no code yet.
    Spec,
    /// Code straight away from the ticket.
    Code,
}

/// A worktree of `repo` on `CU-<id>-<slug>` with `agent` in it, given the whole ticket: as an OpenSpec change
/// to write (`Mode::Spec`) or as work to do (`Mode::Code`). Returns a one-line summary and prints nothing,
/// so a TUI can call it; `progress` gets messages the user must act on (it can wait on them for minutes).
pub fn start(
    cfg: &Config,
    cu: &ClickUp,
    task: &Task,
    agent: &str,
    repo: &str,
    mode: Mode,
    progress: &dyn Fn(String),
) -> Result<String> {
    if agent.is_empty() {
        bail!(NO_AGENT);
    }
    if repo.is_empty() {
        bail!(NO_REPO);
    }
    if mode == Mode::Spec && !openspec::is_installed() {
        bail!(openspec::NOT_INSTALLED);
    }
    // Fail on a broken state dir before anything is created, not after the worktree exists.
    store::update(|_| ())?;
    store::remember("agent", agent);
    store::remember("repo", repo);
    let ticket = cu.ticket(&task.id)?;
    let branch = branch_name(task);
    let wt = herdr::create_worktree(repo, &branch, &task.name)?;
    let mut notes = Vec::new();

    let (prompt, change) = match mode {
        Mode::Code => (code_prompt(&ticket), None),
        Mode::Spec => {
            let dir = Path::new(&wt.path);
            if openspec::ensure_init(dir, agent)? {
                notes.push("set up OpenSpec in the worktree (uncommitted)".to_string());
            }
            let change = branch.to_lowercase();
            let change_dir = openspec::new_change(dir, &change, &task.name)?;
            let ticket_file = change_dir.join("clickup.md");
            fs::write(&ticket_file, ticket_markdown(&ticket))?;
            let rel = ticket_file.strip_prefix(dir).unwrap_or(&ticket_file).display().to_string();
            (spec_prompt(task, &change, &rel), Some(change))
        }
    };

    // Linked before the agent starts, so the hook sees its first status change.
    let link = Link { task_id: task.id.clone(), branch: branch.clone(), change: change.clone(), working: false, reported: false };
    store::update(|state| state.panes.insert(wt.pane_id.clone(), link))?;
    if let Err(e) = herdr::start_agent(&agent_name(task), agent, &wt.pane_id) {
        // Usually the agent asking whether to trust the new worktree folder: the user's call, not ours.
        if !format!("{e:#}").contains("agent_not_ready") {
            return Err(e);
        }
        progress(format!("{agent} is asking something in workspace {} (e.g. trust this folder?) — answer it there; the task is sent once it is ready", wt.workspace_id));
        herdr::wait_until_idle(&wt.pane_id, READY_TIMEOUT_MS)?;
    }
    herdr::prompt_agent(&wt.pane_id, &prompt)?;

    let status = match mode {
        Mode::Spec => cfg.status_spec.as_deref(),
        Mode::Code => cfg.status_started.as_deref(),
    };
    if let Some(warning) = set_status(cu, &task.id, status) {
        notes.push(warning);
    }
    let name = repo.rsplit('/').next().unwrap_or(repo);
    let mut summary = match &change {
        Some(c) => format!("{agent} is writing OpenSpec change {c} in {name}"),
        None => format!("{agent} started on {branch} in {name}"),
    };
    if !notes.is_empty() {
        summary.push_str(&format!(" ({})", notes.join("; ")));
    }
    herdr::notify("ClickUp task started", &format!("{} → {agent} in {}", task.name, wt.workspace_id));
    Ok(summary)
}

/// Opens `url` in the default browser without waiting for it.
pub fn open_url(url: &str) -> Result<()> {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    Command::new(opener).arg(url).spawn()?;
    Ok(())
}

/// Status names differ per ClickUp space, so a missing one is a warning, not a failure.
pub fn set_status(cu: &ClickUp, task_id: &str, status: Option<&str>) -> Option<String> {
    let s = status?;
    cu.set_status(task_id, s).err().map(|e| format!("could not set status \"{s}\": {e:#}"))
}
