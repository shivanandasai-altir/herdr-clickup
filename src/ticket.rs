//! The whole ClickUp ticket an agent works from: the task plus checklists, subtasks, attachments and comments.
use crate::task::{Task, parse_task};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Ticket {
    pub task: Task,
    pub tags: Vec<String>,
    /// The parent task's id, when this task is a subtask.
    pub parent: Option<String>,
    /// `(field name, printable value)`, skipping fields whose value is an id or a file list.
    pub custom_fields: Vec<(String, String)>,
    /// `(checklist name, [(item, resolved)])`
    pub checklists: Vec<(String, Vec<(String, bool)>)>,
    /// `(id, name, status)`
    pub subtasks: Vec<(String, String, String)>,
    /// `(title, url)`
    pub attachments: Vec<(String, String)>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub user: String,
    pub date_ms: i64,
    pub text: String,
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// One custom field as `(name, value)`. Values that are ClickUp ids (users, files, links) are skipped:
/// they mean nothing to a reader and an agent cannot resolve them.
fn custom_field(f: &Value) -> Option<(String, String)> {
    let name = text(f, "name");
    match text(f, "type").as_str() {
        "users" | "attachment" | "files" | "relationship" => return None,
        _ => {}
    }
    let options = f.pointer("/type_config/options").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    let option = |id: &str| {
        options
            .iter()
            .find(|o| o.get("id").and_then(Value::as_str) == Some(id))
            .and_then(|o| o.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| id.to_string())
    };
    let value = match f.get("value")? {
        Value::String(s) if s.is_empty() => return None,
        Value::String(s) => option(s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().filter_map(Value::as_str).map(option).collect();
            if parts.is_empty() {
                return None;
            }
            parts.join(", ")
        }
        _ => return None,
    };
    (!name.is_empty() && !value.is_empty()).then_some((name, value))
}

/// A task from `GET /task/{id}?include_subtasks=true` plus its `GET /task/{id}/comment` response.
pub fn parse_ticket(task: &Value, comments: &Value) -> Option<Ticket> {
    Some(Ticket {
        task: parse_task(task)?,
        tags: arr(task, "tags")
            .iter()
            .map(|t| text(t, "name"))
            .filter(|n| !n.is_empty())
            .collect(),
        parent: task
            .get("parent")
            .and_then(Value::as_str)
            .filter(|p| !p.is_empty())
            .map(str::to_string),
        custom_fields: arr(task, "custom_fields")
            .iter()
            .filter_map(custom_field)
            .collect(),
        checklists: arr(task, "checklists")
            .iter()
            .map(|c| {
                let items = arr(c, "items")
                    .iter()
                    .map(|i| {
                        (
                            text(i, "name"),
                            i.get("resolved").and_then(Value::as_bool).unwrap_or(false),
                        )
                    })
                    .collect();
                (text(c, "name"), items)
            })
            .collect(),
        subtasks: arr(task, "subtasks")
            .iter()
            .map(|s| {
                let status = s
                    .pointer("/status/status")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                (text(s, "id"), text(s, "name"), status.to_string())
            })
            .collect(),
        attachments: arr(task, "attachments")
            .iter()
            .map(|a| (text(a, "title"), text(a, "url")))
            .collect(),
        comments: arr(comments, "comments")
            .iter()
            .map(|c| Comment {
                user: c
                    .pointer("/user/username")
                    .and_then(Value::as_str)
                    .unwrap_or("someone")
                    .to_string(),
                date_ms: text(c, "date").parse().unwrap_or_default(),
                text: text(c, "comment_text").trim().to_string(),
            })
            .filter(|c| !c.text.is_empty())
            .collect(),
    })
}

/// The ticket as a Markdown file the agent (and a reviewer) can read; empty sections are left out.
pub fn ticket_markdown(t: &Ticket) -> String {
    let task = &t.task;
    let mut md = format!("# {}\n\n- ClickUp: {}\n", task.name, task.url);
    if !task.status.is_empty() {
        md += &format!("- Status: {}\n", task.status);
    }
    let who: Vec<&str> = task
        .assignees
        .iter()
        .map(|a| a.username.as_str())
        .filter(|n| !n.is_empty())
        .collect();
    if !who.is_empty() {
        md += &format!("- Assignees: {}\n", who.join(", "));
    }
    if !t.tags.is_empty() {
        md += &format!("- Tags: {}\n", t.tags.join(", "));
    }
    if let Some(parent) = &t.parent {
        md += &format!("- Parent task: {parent}\n");
    }
    if !t.custom_fields.is_empty() {
        md += "\n## Custom fields\n\n";
        for (name, value) in &t.custom_fields {
            md += &format!("- {name}: {value}\n");
        }
    }
    match task.description.trim() {
        "" => md += "\n## Description\n\n_No description._\n",
        d => md += &format!("\n## Description\n\n{d}\n"),
    }
    for (name, items) in &t.checklists {
        md += &format!("\n## Checklist: {name}\n\n");
        for (item, done) in items {
            md += &format!("- [{}] {item}\n", if *done { "x" } else { " " });
        }
    }
    if !t.subtasks.is_empty() {
        md += "\n## Subtasks\n\n";
        for (id, name, status) in &t.subtasks {
            md += &format!("- {name} ({status}) — {id}\n");
        }
    }
    if !t.attachments.is_empty() {
        md += "\n## Attachments\n\n";
        for (title, url) in &t.attachments {
            md += &format!("- [{title}]({url})\n");
        }
    }
    if !t.comments.is_empty() {
        md += "\n## Comments\n";
        for c in &t.comments {
            let quoted: Vec<String> = c
                .text
                .lines()
                .map(|l| format!("> {l}").trim_end().to_string())
                .collect();
            md += &format!(
                "\n**{}** · {}\n\n{}\n",
                c.user,
                date(c.date_ms),
                quoted.join("\n")
            );
        }
    }
    md
}

/// `YYYY-MM-DD` (UTC) for unix ms, without a date library (Howard Hinnant's civil-from-days).
fn date(ms: i64) -> String {
    let z = ms.div_euclid(86_400_000) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// The first prompt of a spec-first start: write the OpenSpec change, no code.
pub fn spec_prompt(t: &Task, change: &str, ticket_path: &str) -> String {
    format!(
        "Plan ClickUp task {id} ({name}) as the OpenSpec change `{change}`: spec first, no code yet.

The full ticket is in {ticket_path}.

1. Write the change's artifacts in order (proposal, specs, design, tasks), each following `openspec instructions <artifact> --change {change}`.
2. Run `openspec validate {change}` and fix what it reports.
3. Do not write or change any application code. Stop when the change validates, and list the open questions and assumptions for me to review.",
        id = t.id,
        name = t.name,
    )
}

/// The first prompt of a start that codes straight away: the task with the whole ticket inline.
pub fn code_prompt(t: &Ticket) -> String {
    format!(
        "Work on ClickUp task {}. The full ticket:\n\n{}",
        t.task.id,
        ticket_markdown(t)
    )
}

/// The prompt that starts the implementation of an approved OpenSpec change.
pub fn apply_prompt(change: &str) -> String {
    format!(
        "Implement the approved OpenSpec change `{change}`.

1. Run `openspec instructions apply --change {change}` and follow those steps.
2. Keep the change's tasks up to date as you complete them.
3. Run the project's tests before you finish."
    )
}
