use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A pane that `start` launched an agent in, and the ClickUp task it works on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub task_id: String,
    pub branch: String,
    /// The agent has been seen working, so a later idle/done is the end of its turn.
    #[serde(default)]
    pub working: bool,
    #[serde(default)]
    pub reported: bool,
}

/// Plugin state: pane id → link. Persisted as JSON in the plugin state dir.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub panes: BTreeMap<String, Link>,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Nothing,
    /// The agent took the prompt: from now on idle/done means it finished.
    MarkWorking { pane_id: String },
    /// The agent finished: tell ClickUp, then mark the link reported.
    Report { pane_id: String, task_id: String, branch: String },
    /// The pane is gone: drop its link.
    Forget { pane_id: String },
    /// The workspace's worktree was removed: drop every link to its panes.
    ForgetWorkspace { workspace_id: String },
}

/// What to do about one herdr event (`{ event, data: { pane_id, agent_status, .. } }`).
/// The event's name with dots turned into underscores: herdr sends `pane_agent_status_changed` while
/// manifests subscribe with `pane.agent_status_changed`.
fn event_name(event: &Value) -> String {
    event.get("event").and_then(Value::as_str).unwrap_or_default().replace('.', "_")
}

pub fn decide(event: &Value, state: &State) -> Decision {
    let name = event_name(event);
    if name == "worktree_removed" {
        let Some(ws) = event.pointer("/data/workspace_id").and_then(Value::as_str) else {
            return Decision::Nothing;
        };
        // Pane ids are `<workspace>:<pane>`.
        let linked = state.panes.keys().any(|p| p.split(':').next() == Some(ws));
        return if linked { Decision::ForgetWorkspace { workspace_id: ws.to_string() } } else { Decision::Nothing };
    }
    let Some(pane_id) = event.pointer("/data/pane_id").and_then(Value::as_str) else {
        return Decision::Nothing;
    };
    let Some(link) = state.panes.get(pane_id) else {
        return Decision::Nothing;
    };
    let status = event.pointer("/data/agent_status").and_then(Value::as_str);
    match name.as_str() {
        "pane_agent_status_changed" if status == Some("working") && !link.working => {
            Decision::MarkWorking { pane_id: pane_id.to_string() }
        }
        // idle is a finish herdr has already shown the user; done is one nobody has seen yet.
        "pane_agent_status_changed" if matches!(status, Some("idle" | "done")) && link.working && !link.reported =>
        {
            Decision::Report {
            pane_id: pane_id.to_string(),
            task_id: link.task_id.clone(),
                branch: link.branch.clone(),
            }
        }
        "pane_closed" => Decision::Forget { pane_id: pane_id.to_string() },
        _ => Decision::Nothing,
    }
}
