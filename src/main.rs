use anyhow::{Result, anyhow, bail};
use dialoguer::{FuzzySelect, MultiSelect, Select};
use herdr_clickup::{
    app::{self, Mode, NO_AGENT, NO_REPO, agent_options, client, open_url, repo_options},
    board,
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::{Decision, Phase, decide, proposed_update},
    link::task_id_from_url,
    report, store,
    task::Task,
    taskspace,
};
use std::{env, io::BufRead};

const USAGE: &str = "usage: herdr-clickup <board|tasks|link|picker|task|list|start [--code] <id> [agent] [repo[,repo]]|hook>";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or_default();
    let in_pane = matches!(cmd, "board-pane" | "picker" | "task");
    if let Err(e) = run(cmd, &args[1.min(args.len())..]) {
        eprintln!("herdr-clickup: {e:#}");
        if in_pane {
            // A pane closes when the process exits; keep the error on screen.
            eprintln!("\nPress Enter to close.");
            let _ = std::io::stdin().lock().lines().next();
        }
        std::process::exit(1);
    }
}

fn run(cmd: &str, rest: &[String]) -> Result<()> {
    match cmd {
        // Actions: open a pane, then exit so herdr is not held up.
        "board" => herdr::open_pane("board", &[]),
        "tasks" => herdr::open_pane("picker", &[]),
        "link" => {
            let url = env::var("HERDR_PLUGIN_CLICKED_URL").unwrap_or_default();
            let id =
                task_id_from_url(&url).ok_or_else(|| anyhow!("not a ClickUp task link: {url}"))?;
            herdr::open_pane("task", &[format!("CLICKUP_TASK_ID={id}")])
        }
        // Panes.
        "board-pane" => board::run(),
        "picker" => picker(),
        "task" => {
            let id =
                env::var("CLICKUP_TASK_ID").map_err(|_| anyhow!("CLICKUP_TASK_ID is not set"))?;
            let (cfg, cu) = client()?;
            let task = cu.task(&id)?;
            println!(
                "{}  [{}]\n{}\n\n{}\n",
                task.name,
                task.status,
                task.url,
                task.description.trim()
            );
            task_menu(&cfg, &cu, &task)
        }
        // Plumbing.
        "list" => {
            let (cfg, cu) = client()?;
            for t in cu.my_tasks(&cfg.space_ids)? {
                println!("{}\t[{}]\t{}", t.id, t.status, t.name);
            }
            Ok(())
        }
        "start" => {
            // Spec first unless --code.
            let mode = if rest.first().map(String::as_str) == Some("--code") {
                Mode::Code
            } else {
                Mode::Spec
            };
            let rest = if mode == Mode::Code { &rest[1..] } else { rest };
            let id = rest
                .first()
                .ok_or_else(|| anyhow!("start needs a task id"))?;
            let (cfg, cu) = client()?;
            let agent = match rest.get(1) {
                Some(a) => a.clone(),
                None => agent_options(&cfg)
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!(NO_AGENT))?,
            };
            let repos: Vec<String> = match rest.get(2) {
                Some(r) => r
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
                None => repo_options().into_iter().take(1).collect(),
            };
            if repos.is_empty() {
                bail!(NO_REPO);
            }
            println!(
                "{}",
                app::start(
                    &cfg,
                    &cu,
                    &cu.task(id)?,
                    &agent,
                    &repos,
                    mode,
                    &|m| println!("{m}")
                )?
            );
            Ok(())
        }
        "hook" => hook(),
        _ => bail!("{USAGE}"),
    }
}

fn picker() -> Result<()> {
    let (cfg, cu) = client()?;
    println!("Loading your ClickUp tasks…");
    let tasks = cu.my_tasks(&cfg.space_ids)?;
    if tasks.is_empty() {
        bail!("no open tasks are assigned to you");
    }
    let items: Vec<String> = tasks
        .iter()
        .map(|t| format!("[{}] {}", t.status, t.name))
        .collect();
    let Some(i) = FuzzySelect::new()
        .with_prompt("Task")
        .items(&items)
        .default(0)
        .interact_opt()?
    else {
        return Ok(());
    };
    task_menu(&cfg, &cu, &tasks[i])
}

fn task_menu(cfg: &Config, cu: &ClickUp, task: &Task) -> Result<()> {
    let choices = [
        "Write the spec first (OpenSpec)",
        "Start coding now",
        "Open in browser",
        "Cancel",
    ];
    let mode = match Select::new()
        .with_prompt(&task.name)
        .items(choices)
        .default(0)
        .interact_opt()?
    {
        Some(0) => Mode::Spec,
        Some(1) => Mode::Code,
        Some(2) => return open_url(&task.url),
        _ => return Ok(()),
    };
    let agents = agent_options(cfg);
    if agents.is_empty() {
        bail!(NO_AGENT);
    }
    let Some(a) = Select::new()
        .with_prompt("Agent")
        .items(&agents)
        .default(0)
        .interact_opt()?
    else {
        return Ok(());
    };
    let repos = repo_options();
    if repos.is_empty() {
        bail!(NO_REPO);
    }
    let defaults: Vec<bool> = (0..repos.len()).map(|i| i == 0).collect();
    let Some(picked) = MultiSelect::new()
        .with_prompt("Repos (space toggles)")
        .items(&repos)
        .defaults(&defaults)
        .interact_opt()?
    else {
        return Ok(());
    };
    if picked.is_empty() {
        return Ok(());
    }
    let selected: Vec<String> = picked.into_iter().map(|i| repos[i].clone()).collect();
    let names: Vec<String> = selected.iter().map(|r| taskspace::repo_name(r)).collect();
    println!("Starting {} in {}…", agents[a], names.join(", "));
    println!(
        "{}",
        app::start(cfg, cu, task, &agents[a], &selected, mode, &|m| println!(
            "{m}"
        ))?
    );
    Ok(())
}

/// Tracks the agents `start` launched. It never writes to ClickUp: when an agent finishes, it drafts an
/// update for the user to approve on the board (`u`) and shows a notification.
fn hook() -> Result<()> {
    let event = serde_json::from_str(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default())
        .unwrap_or(serde_json::Value::Null);
    // Only status names are needed here, and only to draft the proposal.
    let cfg = Config::load().ok();
    // The work summary reads git and files, so build it outside the state lock.
    let summary = if matches!(decide(&event, &store::read()), Decision::Report { .. }) {
        event
            .pointer("/data/pane_id")
            .and_then(serde_json::Value::as_str)
            .and_then(|pane| store::read().panes.get(pane).cloned())
            .map(|link| report::work_summary(&link))
            .unwrap_or_default()
    } else {
        String::new()
    };
    // Decide and claim under the lock, so two hooks for the same finish cannot both draft it.
    let decision = store::update(|state| {
        let d = decide(&event, state);
        match &d {
            Decision::Nothing => {}
            Decision::MarkWorking { pane_id } => state
                .panes
                .get_mut(pane_id)
                .into_iter()
                .for_each(|l| l.working = true),
            Decision::Report {
                pane_id,
                branch,
                change,
                phase,
                ..
            } => {
                let update = proposed_update(
                    branch,
                    change.as_deref(),
                    *phase,
                    cfg.as_ref().and_then(|c| c.status_done.as_deref()),
                    cfg.as_ref().and_then(|c| c.status_spec.as_deref()),
                    &summary,
                );
                if let Some(l) = state.panes.get_mut(pane_id) {
                    l.reported = true;
                    l.reported_as();
                    l.pending = Some(update);
                }
            }
            Decision::Forget { pane_id } => drop(state.panes.remove(pane_id)),
            Decision::ForgetWorkspace { workspace_id } => state
                .panes
                .retain(|pane, _| pane.split(':').next() != Some(workspace_id.as_str())),
        }
        d
    })?;
    if let Decision::Report {
        task_id,
        change,
        phase,
        ..
    } = decision
    {
        let what = match (change.is_some(), phase) {
            (false, _) => "agent finished",
            (true, Phase::Implementing | Phase::Done) => "implementation done",
            (true, _) => "spec drafted",
        };
        herdr::notify(
            &format!("ClickUp {task_id}: {what}"),
            "A ClickUp update is drafted. Review it on the ClickUp board (u); nothing is posted until you approve it.",
        );
    }
    Ok(())
}
