use serde_json::Value;

/// The slice of a ClickUp task the plugin uses.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub status: String,
    pub url: String,
    pub description: String,
}

fn text(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// One task object, as returned by `GET /task/{id}` or inside a task list.
pub fn parse_task(v: &Value) -> Option<Task> {
    let id = text(v, "id");
    let name = text(v, "name");
    if id.is_empty() || name.is_empty() {
        return None;
    }
    // text_content is the plain-text form; description is the older field.
    let description = match text(v, "text_content") {
        t if t.is_empty() => text(v, "description"),
        t => t,
    };
    Some(Task {
        id,
        name,
        status: v.pointer("/status/status").and_then(Value::as_str).unwrap_or_default().to_string(),
        url: text(v, "url"),
        description,
    })
}

/// A `{ "tasks": [...] }` response, skipping entries that are not tasks.
pub fn parse_task_list(v: &Value) -> Vec<Task> {
    v.get("tasks")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(parse_task).collect())
        .unwrap_or_default()
}

const SLUG_MAX: usize = 40;

/// `CU-<id>-<slug>`: lowercase ASCII words joined by dashes, cut on a word boundary.
pub fn branch_name(t: &Task) -> String {
    let words: Vec<String> = t
        .name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut slug = String::new();
    for w in words {
        let extra = if slug.is_empty() { w.len() } else { w.len() + 1 };
        if slug.len() + extra > SLUG_MAX {
            break;
        }
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.push_str(&w);
    }
    if slug.is_empty() { format!("CU-{}", t.id) } else { format!("CU-{}-{}", t.id, slug) }
}

/// The first prompt the agent gets in the task's worktree.
pub fn agent_prompt(t: &Task) -> String {
    let head = format!("Work on ClickUp task {}: {}\n{}", t.id, t.name, t.url);
    match t.description.trim() {
        "" => head,
        d => format!("{head}\n\n{d}"),
    }
}

/// `cu-<id>`, fitted to herdr's agent name rule `[a-z][a-z0-9_-]{0,31}`.
pub fn agent_name(t: &Task) -> String {
    let id: String = t.id.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_lowercase()).collect();
    format!("cu-{id}").chars().take(32).collect()
}
