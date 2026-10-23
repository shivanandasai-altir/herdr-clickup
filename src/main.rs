use anyhow::{anyhow, bail, Result};
use dialoguer::{FuzzySelect, Select};
use herdr_clickup::{
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::{decide, Decision, Link},
    link::task_id_from_url,
    store,
    task::{agent_name, agent_prompt, branch_name, Task},
};
use std::{env, io::BufRead, process::Command};

const USAGE: &str = "usage: herdr-clickup <tasks|link|picker|task|list|start <id>|hook>";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or_default();
    let in_pane = matches!(cmd, "picker" | "task");
    if let Err(e) = run(cmd, &args[1.min(args.len())..]) {
        eprintln!("herdr-clickup: {e:#}");
        if in_pane {
            // A popup closes when the process exits; keep the error on screen.
            eprintln!("\nPress Enter to close.");
            let _ = std::io::stdin().lock().lines().next();
        }
        std::process::exit(1);
    }
}

fn run(cmd: &str, rest: &[String]) -> Result<()> {
    match cmd {
        // Actions: open a popup pane, then exit so herdr is not held up.
        "tasks" => herdr::open_pane("picker", &[]),
        "link" => {
            let url = env::var("HERDR_PLUGIN_CLICKED_URL").unwrap_or_default();
            let id = task_id_from_url(&url).ok_or_else(|| anyhow!("not a ClickUp task link: {url}"))?;
            herdr::open_pane("task", &[format!("CLICKUP_TASK_ID={id}")])
        }
        // Panes.
        "picker" => picker(),
        "task" => {
            let id = env::var("CLICKUP_TASK_ID").map_err(|_| anyhow!("CLICKUP_TASK_ID is not set"))?;
            let (cfg, cu) = client()?;
            let task = cu.task(&id)?;
            println!("{}  [{}]\n{}\n\n{}\n", task.name, task.status, task.url, task.description.trim());
            task_menu(&cfg, &cu, &task)
        }
        // Plumbing.
        "start" => {
            let id = rest.first().ok_or_else(|| anyhow!("start needs a task id"))?;
            let (cfg, cu) = client()?;
            start(&cfg, &cu, &cu.task(id)?)
        }
        "list" => {
            let (_, cu) = client()?;
            for t in cu.my_tasks()? {
                println!("{}\t[{}]\t{}", t.id, t.status, t.name);
            }
            Ok(())
        }
        "hook" => hook(),
        _ => bail!("{USAGE}"),
    }
}

fn client() -> Result<(Config, ClickUp)> {
    let cfg = Config::load()?;
    let cu = ClickUp::new(cfg.token.clone(), cfg.team_id.clone());
    Ok((cfg, cu))
}

fn picker() -> Result<()> {
    let (cfg, cu) = client()?;
    println!("Loading your ClickUp tasks…");
    let tasks = cu.my_tasks()?;
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
    let start_label = format!("Start: new worktree + {}", cfg.agent);
    let choices = [start_label.as_str(), "Open in browser", "Cancel"];
    match Select::new().with_prompt(&task.name).items(choices).default(0).interact_opt()? {
        Some(0) => start(cfg, cu, task),
        Some(1) => open_url(&task.url),
        _ => Ok(()),
    }
}

/// Worktree on `CU-<id>-<slug>`, an agent in it with the task as its prompt, and the task marked started.
fn start(cfg: &Config, cu: &ClickUp, task: &Task) -> Result<()> {
    let branch = branch_name(task);
    println!("Creating worktree {branch}…");
    let wt = herdr::create_worktree(&branch, &task.name)?;
    // Linked before the agent starts, so the hook sees its first status change.
    let link = Link { task_id: task.id.clone(), branch: branch.clone(), working: false, reported: false };
    store::update(|state| state.panes.insert(wt.pane_id.clone(), link))?;
    println!("Starting {} in {}…", cfg.agent, wt.pane_id);
    herdr::start_agent(&agent_name(task), &cfg.agent, &wt.pane_id)?;
    herdr::prompt_agent(&wt.pane_id, &agent_prompt(task))?;

    set_status(cu, &task.id, cfg.status_started.as_deref());
    herdr::notify("ClickUp task started", &format!("{} → {}", task.name, wt.workspace_id));
    Ok(())
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
        }
        d
    })?;
    if let Decision::Report { task_id, branch, .. } = decision {
        let (cfg, cu) = client()?;
        cu.comment(&task_id, &format!("herdr agent finished its turn on branch `{branch}`."))?;
        set_status(&cu, &task_id, cfg.status_done.as_deref());
        herdr::notify("ClickUp task updated", &format!("{task_id}: agent done on {branch}"));
    }
    Ok(())
}

/// Status names differ per ClickUp space, so a missing one is a warning, not a failure.
fn set_status(cu: &ClickUp, task_id: &str, status: Option<&str>) {
    let Some(s) = status else { return };
    if let Err(e) = cu.set_status(task_id, s) {
        eprintln!("warning: could not set status \"{s}\": {e:#}");
    }
}

fn open_url(url: &str) -> Result<()> {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    Command::new(opener).arg(url).status()?;
    Ok(())
}
