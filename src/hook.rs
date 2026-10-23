use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A pane that `start` launched an agent in, and the ClickUp task it works on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub task_id: String,
    pub branch: String,
    /// The OpenSpec change of a spec-first start; `None` for a start that codes straight away.
    #[serde(default)]
    pub change: Option<String>,
    /// The agent has been seen working, so a later idle/done is the end of its turn.
    #[serde(default)]
    pub working: bool,
    #[serde(default)]
    pub reported: bool,
    /// Where the task is in the spec-first flow. Absent in links written before phases existed.
    #[serde(default)]
    pub phase: Option<Phase>,
    /// A ClickUp update drafted when the agent finished, waiting for the user to approve or discard it.
    #[serde(default)]
    pub pending: Option<Update>,
}

/// The spec-first flow of one task: write the spec, review it, implement it, finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    /// The agent is writing the OpenSpec change; no code yet.
    SpecDrafting,
    /// The change validates and waits for the user to review it.
    SpecDrafted,
    /// The agent implements the approved change.
    Implementing,
    /// The agent finished implementing.
    Done,
}

impl Phase {
    /// Short label for the board's phase column.
    pub fn label(self) -> &'static str {
        match self {
            Phase::SpecDrafting => "spec…",
            Phase::SpecDrafted => "spec ✓",
            Phase::Implementing => "code…",
            Phase::Done => "done",
        }
    }

    /// The phase a report moves the link to.
    fn after_report(self) -> Phase {
        match self {
            Phase::SpecDrafting | Phase::SpecDrafted => Phase::SpecDrafted,
            Phase::Implementing | Phase::Done => Phase::Done,
        }
    }
}

impl Link {
    /// Older links have no phase. A change that was already reported means the spec is drafted; a
    /// change that was not means the spec is still being written; no change means a code start.
    pub fn phase(&self) -> Phase {
        self.phase.unwrap_or(match (self.change.is_some(), self.reported) {
            (true, false) => Phase::SpecDrafting,
            (true, true) => Phase::SpecDrafted,
            (false, _) => Phase::Implementing,
        })
    }

    /// Moves the link past a report the user has not approved yet.
    pub fn reported_as(&mut self) {
        self.phase = Some(self.phase().after_report());
    }

    /// `i`: the agent now implements the approved change; the next finish is an implementation report.
    pub fn implementing(&mut self) {
        self.phase = Some(Phase::Implementing);
        self.working = false;
        self.reported = false;
    }
}

/// A ClickUp change the plugin proposes. It is only ever posted on the user's explicit approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Update {
    pub comment: String,
    pub status: Option<String>,
}

/// What to propose when the agent on `branch` finishes: "spec drafted" for a spec start (`change` set),
/// "finished" plus the done status for a code start.
pub fn proposed_update(
    branch: &str,
    change: Option<&str>,
    phase: Phase,
    status_done: Option<&str>,
    status_spec: Option<&str>,
) -> Update {
    match (change, phase) {
        (Some(c), Phase::SpecDrafting | Phase::SpecDrafted) => Update {
            comment: format!(
                "Spec drafted: OpenSpec change `{c}` on branch `{branch}` is ready for review."
            ),
            status: status_spec.map(str::to_string),
        },
        (Some(c), Phase::Implementing | Phase::Done) => Update {
            comment: format!(
                "Implementation done: OpenSpec change `{c}` on branch `{branch}` is ready for review."
            ),
            status: status_done.map(str::to_string),
        },
        (None, _) => Update {
            comment: format!("herdr agent finished its turn on branch `{branch}`."),
            status: status_done.map(str::to_string),
        },
    }
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
    MarkWorking {
        pane_id: String,
    },
    /// The agent finished: tell ClickUp, then mark the link reported.
    Report {
        pane_id: String,
        task_id: String,
        branch: String,
        change: Option<String>,
        phase: Phase,
    },
    /// The pane is gone: drop its link.
    Forget {
        pane_id: String,
    },
    /// The workspace's worktree was removed: drop every link to its panes.
    ForgetWorkspace {
        workspace_id: String,
    },
}

/// What to do about one herdr event (`{ event, data: { pane_id, agent_status, .. } }`).
/// The event's name with dots turned into underscores: herdr sends `pane_agent_status_changed` while
/// manifests subscribe with `pane.agent_status_changed`.
fn event_name(event: &Value) -> String {
    event
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .replace('.', "_")
}

pub fn decide(event: &Value, state: &State) -> Decision {
    let name = event_name(event);
    if name == "worktree_removed" {
        let Some(ws) = event.pointer("/data/workspace_id").and_then(Value::as_str) else {
            return Decision::Nothing;
        };
        // Pane ids are `<workspace>:<pane>`.
        let linked = state.panes.keys().any(|p| p.split(':').next() == Some(ws));
        return if linked {
            Decision::ForgetWorkspace {
                workspace_id: ws.to_string(),
            }
        } else {
            Decision::Nothing
        };
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
            Decision::MarkWorking {
                pane_id: pane_id.to_string(),
            }
        }
        // idle is a finish herdr has already shown the user; done is one nobody has seen yet.
        "pane_agent_status_changed"
            if matches!(status, Some("idle" | "done")) && link.working && !link.reported =>
        {
            Decision::Report {
                pane_id: pane_id.to_string(),
                task_id: link.task_id.clone(),
                branch: link.branch.clone(),
                change: link.change.clone(),
                phase: link.phase(),
            }
        }
        "pane_closed" => Decision::Forget {
            pane_id: pane_id.to_string(),
        },
        _ => Decision::Nothing,
    }
}
