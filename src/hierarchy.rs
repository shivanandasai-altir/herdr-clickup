use serde_json::Value;

/// A ClickUp list, where it lives, and its sprint dates when it is a sprint.
#[derive(Debug, Clone, PartialEq)]
pub struct ListRef {
    pub id: String,
    pub name: String,
    pub space: String,
    pub folder: Option<String>,
    /// Unix ms; set on sprint lists.
    pub start: Option<i64>,
    pub due: Option<i64>,
    pub task_count: Option<u64>,
}

/// Lists from `GET /space/{id}/folder` (lists nested in folders) and `GET /space/{id}/list` (folderless).
pub fn parse_space_lists(space: &str, folders: &Value, folderless: &Value) -> Vec<ListRef> {
    let in_folders = arr(folders, "folders").iter().flat_map(|f| {
        let folder = f.get("name").and_then(Value::as_str).map(str::to_string);
        arr(f, "lists").iter().filter_map(move |l| list(space, folder.clone(), l))
    });
    let loose = arr(folderless, "lists").iter().filter_map(|l| list(space, None, l));
    in_folders.chain(loose).collect()
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default()
}

/// ClickUp sends numbers as strings in some fields and as numbers in others.
fn num(v: &Value, key: &str) -> Option<i64> {
    match v.get(key)? {
        Value::String(s) => s.parse().ok(),
        n => n.as_i64(),
    }
}

fn list(space: &str, folder: Option<String>, l: &Value) -> Option<ListRef> {
    Some(ListRef {
        id: l.get("id")?.as_str()?.to_string(),
        name: l.get("name")?.as_str()?.to_string(),
        space: space.to_string(),
        folder,
        start: num(l, "start_date"),
        due: num(l, "due_date"),
        task_count: num(l, "task_count").and_then(|n| u64::try_from(n).ok()),
    })
}

/// Sprint lists (both dates set) running at `now_ms`, in the order given.
pub fn current_sprints(lists: &[ListRef], now_ms: i64) -> Vec<&ListRef> {
    lists
        .iter()
        .filter(|l| matches!((l.start, l.due), (Some(s), Some(d)) if s <= now_ms && now_ms <= d))
        .collect()
}
