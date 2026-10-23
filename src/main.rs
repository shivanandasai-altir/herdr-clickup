use anyhow::{anyhow, bail, Result};
use dialoguer::{FuzzySelect, Select};
use herdr_clickup::{
    app::{self, agent_options, client, open_url, repo_options, set_status, NO_AGENT, NO_REPO},
    board,
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::{decide, Decision},
    link::task_id_from_url,
    store,
    task::Task,
};
use std::{env, io::BufRead};

const USAGE: &str = "usage: herdr-clickup <board|tasks|link|picker|task|list|start <id> [agent] [repo]|hook>";

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
            let id = task_id_from_url(&url).ok_or_else(|| anyhow!("not a ClickUp task link: {url}"))?;
            herdr::open_pane("task", &[format!("CLICKUP_TASK_ID={id}")])
        }
        // Panes.
        "board-pane" => board::run(),
        "picker" => picker(),
        "task" => {
            let id = env::var("CLICKUP_TASK_ID").map_err(|_| anyhow!("CLICKUP_TASK_ID is not set"))?;
            let (cfg, cu) = client()?;
            let task = cu.task(&id)?;
            println!("{}  [{}]\n{}\n\n{}\n", task.name, task.status, task.url, task.description.trim());
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
            let id = rest.first().ok_or_else(|| anyhow!("start needs a task id"))?;
            let (cfg, cu) = client()?;
            let agent = match rest.get(1) {
                Some(a) => a.clone(),
                None => agent_options(&cfg).into_iter().next().ok_or_else(|| anyhow!(NO_AGENT))?,
            };
            let repo = match rest.get(2) {
                Some(r) => r.clone(),
                None => repo_options().into_iter().next().ok_or_else(|| anyhow!(NO_REPO))?,
            };
            println!("{}", app::start(&cfg, &cu, &cu.task(id)?, &agent, &repo, &|m| println!("{m}"))?);
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
    let items: Vec<String> = tasks.iter().map(|t| format!("[{}] {}", t.status, t.name)).collect();
    let Some(i) = FuzzySelect::new().with_prompt("Task").items(&items).default(0).interact_opt()? else {
        return Ok(());
    };
    task_menu(&cfg, &cu, &tasks[i])
}

fn task_menu(cfg: &Config, cu: &ClickUp, task: &Task) -> Result<()> {
    let choices = ["Start: new worktree + agent", "Open in browser", "Cancel"];
    match Select::new().with_prompt(&task.name).items(choices).default(0).interact_opt()? {
        Some(0) => {
            let agents = agent_options(cfg);
            if agents.is_empty() {
                bail!(NO_AGENT);
            }
            let Some(a) = Select::new().with_prompt("Agent").items(&agents).default(0).interact_opt()? else {
                return Ok(());
            };
            let repos = repo_options();
            if repos.is_empty() {
                bail!(NO_REPO);
            }
            let Some(r) = Select::new().with_prompt("Repo").items(&repos).default(0).interact_opt()? else {
                return Ok(());
            };
            println!("Starting {} in {}…", agents[a], repos[r]);
            println!("{}", app::start(cfg, cu, task, &agents[a], &repos[r], &|m| println!("{m}"))?);
            Ok(())
        }
        Some(1) => open_url(&task.url),
        _ => Ok(()),
    }
}

fn hook() -> Result<()> {
    let event = serde_json::from_str(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default())
        .unwrap_or(serde_json::Value::Null);
    // Decide and claim under the lock, so two hooks for the same finish cannot both report it.
    let decision = store::update(|state| {
        let d = decide(&event, state);
        match &d {
            Decision::Nothing => {}
            Decision::MarkWorking { pane_id } => state.panes.get_mut(pane_id).into_iter().for_each(|l| l.working = true),
            Decision::Report { pane_id, .. } => state.panes.get_mut(pane_id).into_iter().for_each(|l| l.reported = true),
            Decision::Forget { pane_id } => drop(state.panes.remove(pane_id)),
            Decision::ForgetWorkspace { workspace_id } => {
                state.panes.retain(|pane, _| pane.split(':').next() != Some(workspace_id.as_str()))
            }
        }
        d
    })?;
    if let Decision::Report { task_id, branch, .. } = decision {
        let (cfg, cu) = client()?;
        cu.comment(&task_id, &format!("herdr agent finished its turn on branch `{branch}`."))?;
        if let Some(warning) = set_status(&cu, &task_id, cfg.status_done.as_deref()) {
            eprintln!("warning: {warning}");
        }
        herdr::notify("ClickUp task updated", &format!("{task_id}: agent done on {branch}"));
    }
    Ok(())
}
