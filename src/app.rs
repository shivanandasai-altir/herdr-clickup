//! What the picker, the board and the CLI share: setup, agent choices, and starting a task.
use crate::{
    agents::{agent_choices, is_installed},
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::{Link, Phase, RepoWorktree, Update, start_update},
    openspec, repos, store,
    task::{Task, agent_name, branch_name},
    taskspace,
    ticket::{apply_prompt, code_prompt, spec_prompt, ticket_markdown},
};
use anyhow::{Result, anyhow, bail};
use std::{fs, path::Path, process::Command};

/// How long a start waits for the user to answer an agent's startup question.
const READY_TIMEOUT_MS: u64 = 15 * 60 * 1000;

pub const NO_AGENT: &str = "no supported coding agent is installed";
pub const NO_REPO: &str =
    "no git repo is open in a herdr workspace; open the task's repo in herdr first";

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

/// Starts `agent` on `task` in a worktree of every repo in `repos` on `CU-<id>-<slug>`, given the whole
/// ticket: as an OpenSpec change to write (`Mode::Spec`) or as work to do (`Mode::Code`). One repo keeps
/// today's single-worktree start; two or more share a task folder. Returns a one-line summary and prints
/// nothing, so a TUI can call it; `progress` gets messages the user must act on.
pub fn start(
    cfg: &Config,
    cu: &ClickUp,
    task: &Task,
    agent: &str,
    repos: &[String],
    mode: Mode,
    progress: &dyn Fn(String),
) -> Result<String> {
    if agent.is_empty() {
        bail!(NO_AGENT);
    }
    if repos.is_empty() || repos.iter().any(|r| r.is_empty()) {
        bail!(NO_REPO);
    }
    if mode == Mode::Spec && !openspec::is_installed() {
        bail!(openspec::NOT_INSTALLED);
    }
    // Fail on a broken state dir before anything is created, not after a worktree exists.
    store::update(|_| ())?;
    store::remember("agent", agent);
    store::remember("repo", &repos[0]);
    let ticket = cu.ticket(&task.id)?;
    let branch = branch_name(task);
    let multi = repos.len() > 1;
    let mut notes = Vec::new();

    // One workspace and one agent. A multi-repo spec start creates only the host worktree now;
    // implementation adds the repos its approved repos.txt lists.
    let (workspace_id, pane_id, dir, worktrees, host) = if multi {
        let dir = taskspace::task_dir(&branch)?;
        let now = if mode == Mode::Spec {
            &repos[..1]
        } else {
            repos
        };
        let worktrees = taskspace::add_worktrees(&dir, now, &branch, &[])?;
        let dir_text = dir.display().to_string();
        let (workspace_id, pane_id) = herdr::create_workspace(&dir_text, &task.name)?;
        let host = worktrees[0].path.clone();
        (workspace_id, pane_id, dir_text, worktrees, host)
    } else {
        let wt = herdr::create_worktree(&repos[0], &branch, &task.name)?;
        let worktree = RepoWorktree {
            repo: repos[0].clone(),
            path: wt.path.clone(),
        };
        (
            wt.workspace_id,
            wt.pane_id,
            wt.path.clone(),
            vec![worktree],
            wt.path,
        )
    };

    let (prompt, change) = match mode {
        Mode::Code => (code_prompt(&ticket), None),
        Mode::Spec => {
            let host_dir = Path::new(&host);
            if openspec::ensure_init(host_dir, agent)? {
                notes.push("set up OpenSpec in the worktree (uncommitted)".to_string());
            }
            let change = branch.to_lowercase();
            let change_dir = openspec::new_change(host_dir, &change, &task.name)?;
            fs::write(change_dir.join("clickup.md"), ticket_markdown(&ticket))?;
            let dir_path = Path::new(&dir);
            let ticket_rel = relative(dir_path, &change_dir.join("clickup.md"));
            let mut prompt = spec_prompt(task, &change, &ticket_rel);
            // A single-repo start keeps today's worktree and has no list to grow.
            if multi {
                taskspace::write_repos(&change_dir, repos)?;
                let repos_rel = relative(dir_path, &change_dir.join("repos.txt"));
                let host_rel = relative(dir_path, Path::new(&host));
                if host_rel != "." {
                    prompt.push_str(&format!(
                        "\n\nThe OpenSpec change lives in `{host_rel}`. Run every `openspec` command from `{host_rel}`."
                    ));
                }
                prompt.push_str(&format!(
                    "\n\nThe repos this task can touch are in {repos_rel}: one absolute path per line. Edit that list so it holds exactly the repos the change affects; implementation creates a worktree for each. You may read the other repos listed there to judge impact, but do not change code in any repo."
                ));
            }
            (prompt, Some(change))
        }
    };

    if multi {
        taskspace::write_agents_md(
            Path::new(&dir),
            &task.name,
            &task.url,
            &branch,
            &worktrees,
            agent,
        )?;
    }

    // Linked before the agent starts, so the hook sees its first status change.
    let phase = match mode {
        Mode::Spec => Phase::SpecDrafting,
        Mode::Code => Phase::Implementing,
    };
    let pending = start_update(match mode {
        Mode::Spec => cfg.status_spec.as_deref(),
        Mode::Code => cfg.status_started.as_deref(),
    });
    if let Some(status) = pending.as_ref().and_then(|u| u.status.as_deref()) {
        notes.push(format!("status \"{status}\" is drafted for approval"));
    }
    let link = Link {
        task_id: task.id.clone(),
        branch: branch.clone(),
        change: change.clone(),
        phase: Some(phase),
        dir: Some(dir.clone()),
        worktrees: worktrees.clone(),
        title: task.name.clone(),
        url: task.url.clone(),
        agent: agent.to_string(),
        working: false,
        reported: false,
        pending,
    };
    store::update(|state| state.panes.insert(pane_id.clone(), link))?;
    if let Err(e) = herdr::start_agent(&agent_name(task), agent, &pane_id) {
        // Usually the agent asking whether to trust the new worktree folder: the user's call, not ours.
        if !format!("{e:#}").contains("agent_not_ready") {
            return Err(e);
        }
        progress(format!(
            "{agent} is asking something in workspace {workspace_id} (e.g. trust this folder?) — answer it there; the task is sent once it is ready"
        ));
        herdr::wait_until_idle(&pane_id, READY_TIMEOUT_MS)?;
    }
    herdr::prompt_agent(&pane_id, &prompt)?;

    let names: Vec<String> = worktrees
        .iter()
        .map(|w| taskspace::repo_name(&w.repo))
        .collect();
    let name = match names.as_slice() {
        [one] => one.clone(),
        many => format!("{} repos", many.len()),
    };
    let mut summary = match &change {
        Some(c) => format!("{agent} is writing OpenSpec change {c} in {name}"),
        None => format!("{agent} started on {branch} in {name}"),
    };
    if !notes.is_empty() {
        summary.push_str(&format!(" ({})", notes.join("; ")));
    }
    herdr::notify(
        "ClickUp task started",
        &format!("{} → {agent} in {}", task.name, workspace_id),
    );
    Ok(summary)
}

/// Sends the apply workflow of an approved change to the agent in `pane_id`, adds the worktrees its
/// `repos.txt` lists, and makes the next finish report the implementation rather than the spec.
pub fn implement(pane_id: &str, change: &str) -> Result<String> {
    let link = store::read()
        .panes
        .get(pane_id)
        .cloned()
        .ok_or_else(|| anyhow!("no task link for pane {pane_id}"))?;
    let host = link
        .worktrees
        .first()
        .map(|w| w.path.clone())
        .or_else(|| link.dir.clone());
    if let Some(host) = &host {
        // A change that does not validate is not approved, however the board got here.
        openspec::validate(Path::new(host), change)?;
    }
    // The link's `working` flag is stale after a report; ask herdr for the live state.
    if !matches!(herdr::agent_status(pane_id)?.as_str(), "idle" | "done") {
        bail!("the agent is still working; wait for it to finish before implementing");
    }
    let folder = link.folder().map(str::to_string);
    let mut all = link.worktrees.clone();
    if let (Some(folder), Some(host)) = (folder.as_deref(), link.worktrees.first()) {
        let change_dir = Path::new(&host.path)
            .join("openspec")
            .join("changes")
            .join(change);
        let repos = taskspace::read_repos(&change_dir)?;
        for repo in repos {
            let added = taskspace::add_worktrees(
                Path::new(folder),
                std::slice::from_ref(&repo),
                &link.branch,
                &all,
            )?;
            if added.is_empty() {
                continue;
            }
            all.extend(added.clone());
            // Record each new worktree at once, so a later failure still lets close remove it.
            store::update(|state| {
                if let Some(l) = state.panes.get_mut(pane_id) {
                    l.worktrees.extend(added);
                }
            })?;
        }
        taskspace::write_agents_md(
            Path::new(folder),
            &link.title,
            &link.url,
            &link.branch,
            &all,
            &link.agent,
        )?;
    }
    // Claim the next finish while the link is still reviewed; a second `i` claims nothing.
    let claimed = store::update(|state| {
        let Some(l) = state.panes.get_mut(pane_id) else {
            return false;
        };
        if !l.can_implement() {
            return false;
        }
        l.worktrees = all.clone();
        l.implementing();
        true
    })?;
    if !claimed {
        bail!(
            "the spec agent is still working, or the change is no longer ready; try again when it is idle"
        );
    }
    let mut prompt = apply_prompt(change);
    if let (Some(folder), Some(host)) = (folder.as_deref(), link.worktrees.first()) {
        let host_rel = relative(Path::new(folder), Path::new(&host.path));
        if host_rel != "." {
            prompt.push_str(&format!(
                "\n\nRun every `openspec` command from `{host_rel}`, the repo that holds the change."
            ));
        }
    }
    if let Err(e) = herdr::prompt_agent(pane_id, &prompt) {
        let _ = store::update(|state| {
            if let Some(l) = state.panes.get_mut(pane_id) {
                l.phase = Some(Phase::SpecDrafted);
                l.reported = true;
            }
        });
        return Err(e);
    }
    Ok(format!("sent the apply workflow for {change}"))
}

/// Adds a worktree of `repo` to a running multi-repo task, and to its spec's repo list when it has one.
pub fn add_repo(task_id: &str, repo: &str) -> Result<String> {
    let (pane, link) = find_link(task_id)?;
    let folder = link
        .folder()
        .ok_or_else(|| anyhow!("{} has no task folder; start it with 2+ repos", link.branch))?;
    let added = taskspace::add_worktrees(
        Path::new(folder),
        &[repo.to_string()],
        &link.branch,
        &link.worktrees,
    )?;
    let Some(added) = added.into_iter().next() else {
        bail!("{} is already in this task", taskspace::repo_name(repo));
    };
    let mut all = link.worktrees.clone();
    all.push(added.clone());
    // Record it first, so a later failure still lets close remove the checkout.
    store::update(|state| {
        if let Some(l) = state.panes.get_mut(&pane) {
            l.worktrees.push(added.clone());
        }
    })?;
    taskspace::write_agents_md(
        Path::new(folder),
        &link.title,
        &link.url,
        &link.branch,
        &all,
        &link.agent,
    )?;
    if let (Some(host), Some(change)) = (all.first(), link.change.as_deref()) {
        let change_dir = Path::new(&host.path)
            .join("openspec")
            .join("changes")
            .join(change);
        if change_dir.is_dir() {
            taskspace::add_repo(&change_dir, repo)?;
        }
    }
    Ok(format!(
        "added {} at {}",
        taskspace::repo_name(repo),
        added.path
    ))
}

/// What closing a task would remove, and which repos hold uncommitted work.
pub struct ClosePreview {
    pub branch: String,
    pub folder: Option<String>,
    pub worktrees: Vec<RepoWorktree>,
    pub dirty: Vec<String>,
}

pub fn close_preview(task_id: &str) -> Result<ClosePreview> {
    let (_, link) = find_link(task_id)?;
    let mut dirty = taskspace::dirty(&link.worktrees);
    // Links written before worktrees were stored only have their checkout folder.
    if link.worktrees.is_empty()
        && let Some(dir) = &link.dir
        && taskspace::dirty_dir(dir)
    {
        dirty.push(dir.clone());
    }
    Ok(ClosePreview {
        branch: link.branch.clone(),
        folder: link.folder().map(str::to_string),
        dirty,
        worktrees: link.worktrees.clone(),
    })
}

/// Removes every worktree and the task folder of a task, and closes its workspace. Uncommitted work is
/// discarded, which is why the board asks first and shows what is dirty.
pub fn close_task(task_id: &str) -> Result<String> {
    let panes: Vec<(String, Link)> = store::read()
        .panes
        .into_iter()
        .filter(|(_, l)| l.task_id == task_id)
        .collect();
    if panes.is_empty() {
        bail!("no open task {task_id}");
    }
    for (pane, link) in panes {
        let workspace = pane.split(':').next().unwrap_or_default().to_string();
        if let Some(folder) = link.folder() {
            let failed = taskspace::remove_worktrees(&link.worktrees);
            if !failed.is_empty() {
                bail!("could not remove every worktree: {}", failed.join("; "));
            }
            if let Err(e) = fs::remove_dir_all(folder) {
                // A folder removed by hand is already closed.
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(anyhow!("removing task folder {folder}: {e}"));
                }
            }
            let _ = herdr::close_workspace(&workspace);
        } else {
            herdr::remove_worktree_workspace(&workspace)?;
            let _ = herdr::close_workspace(&workspace);
        }
        store::update(|state| state.panes.remove(&pane))?;
    }
    Ok(format!(
        "closed task {task_id}: removed its worktrees and workspace"
    ))
}

fn find_link(task_id: &str) -> Result<(String, Link)> {
    store::read()
        .panes
        .into_iter()
        .find(|(_, l)| l.task_id == task_id)
        .ok_or_else(|| anyhow!("no task link for {task_id}"))
}

/// `path` relative to `dir` when it is inside it, for prompts the agent reads from `dir`.
fn relative(dir: &Path, path: &Path) -> String {
    path.strip_prefix(dir).unwrap_or(path).display().to_string()
}

/// Opens `url` in the default browser without waiting for it.
pub fn open_url(url: &str) -> Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener).arg(url).spawn()?;
    Ok(())
}

/// Posts an update the user approved: the comment, then the status. Called only from an explicit user action.
pub fn post_update(cu: &ClickUp, task_id: &str, update: &Update) -> Result<String> {
    let mut summary = String::new();
    if !update.comment.trim().is_empty() {
        cu.comment(task_id, &update.comment)?;
        summary.push_str(&format!("posted the comment on {task_id}"));
    }
    if let Some(warning) = set_status(cu, task_id, update.status.as_deref()) {
        if !summary.is_empty() {
            summary.push_str(", ");
        }
        summary.push_str(&warning);
    } else if let Some(s) = &update.status {
        if !summary.is_empty() {
            summary.push_str(" and ");
        }
        summary.push_str(&format!("set it to {s}"));
    }
    if summary.is_empty() {
        summary = format!("nothing to post on {task_id}");
    }
    clear_pending(task_id)?;
    Ok(summary)
}

/// The text `$EDITOR` opens: the comment, then the status line. The header is dropped on read.
pub fn edit_text(update: &Update) -> String {
    format!(
        "# herdr-clickup draft: edit the comment, then the status line. An empty comment posts only the status.\n\n{}\n\nstatus: {}\n",
        update.comment.trim_end(),
        update.status.as_deref().unwrap_or_default()
    )
}

/// The draft after an edit. The last `status:` line sets the status, and everything before it is
/// the comment. A missing or empty status line means no status change.
pub fn parse_edit(text: &str) -> Update {
    let lines: Vec<&str> = text.lines().collect();
    let start = usize::from(
        lines
            .first()
            .is_some_and(|l| l.starts_with("# herdr-clickup draft:")),
    );
    let status_at = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("status:"));
    let (comment_lines, status) = match status_at {
        Some(i) => (
            &lines[start.min(i)..i],
            Some(
                lines[i]
                    .trim_start()
                    .trim_start_matches("status:")
                    .trim()
                    .to_string(),
            ),
        ),
        None => (&lines[start..], None),
    };
    Update {
        comment: comment_lines.join("\n").trim().to_string(),
        status: status.filter(|s| !s.is_empty()),
    }
}

/// Saves an edited draft, so the board and the hook see it.
pub fn save_pending(task_id: &str, update: &Update) -> Result<()> {
    store::update(|state| {
        state
            .panes
            .values_mut()
            .filter(|l| l.task_id == task_id)
            .for_each(|l| l.pending = Some(update.clone()))
    })
}

/// Drops the drafted update of every link to `task_id`.
pub fn clear_pending(task_id: &str) -> Result<()> {
    store::update(|state| {
        state
            .panes
            .values_mut()
            .filter(|l| l.task_id == task_id)
            .for_each(|l| l.pending = None)
    })
}

/// Status names differ per ClickUp space, so a missing one is a warning, not a failure.
pub fn set_status(cu: &ClickUp, task_id: &str, status: Option<&str>) -> Option<String> {
    let s = status?;
    cu.set_status(task_id, s)
        .err()
        .map(|e| format!("could not set status \"{s}\": {e:#}"))
}
