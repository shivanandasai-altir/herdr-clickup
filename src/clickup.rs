use crate::hierarchy::{parse_space_lists, ListRef};
use crate::task::{parse_task, parse_task_list, Task};
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

const API: &str = "https://api.clickup.com/api/v2";

/// Thin blocking client for the ClickUp v2 REST API.
#[derive(Clone)]
pub struct ClickUp {
    token: String,
    team_id: Option<String>,
}

impl ClickUp {
    pub fn new(token: String, team_id: Option<String>) -> Self {
        ClickUp { token, team_id }
    }

    fn get(&self, path: &str) -> Result<Value> {
        ureq::get(&format!("{API}{path}"))
            .header("Authorization", &self.token)
            .call()
            .with_context(|| format!("GET {path}"))?
            .body_mut()
            .read_json()
            .with_context(|| format!("GET {path}: bad JSON"))
    }

    fn send(&self, method: &str, path: &str, body: Value) -> Result<()> {
        let url = format!("{API}{path}");
        let req = match method {
            "PUT" => ureq::put(&url),
            _ => ureq::post(&url),
        };
        req.header("Authorization", &self.token)
            .send_json(&body)
            .with_context(|| format!("{method} {path}"))?;
        Ok(())
    }

    pub fn team_id(&self) -> Result<String> {
        if let Some(t) = &self.team_id {
            return Ok(t.clone());
        }
        let teams = self.get("/team")?;
        teams
            .pointer("/teams/0/id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("this token can see no ClickUp workspace"))
    }

    pub fn my_user_id(&self) -> Result<u64> {
        self.get("/user")?
            .pointer("/user/id")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("GET /user returned no user id"))
    }

    /// Every page of a task query (ClickUp pages hold 100), up to `MAX_PAGES`.
    fn all_tasks(&self, path: &str) -> Result<Vec<Task>> {
        const MAX_PAGES: u32 = 20;
        let mut tasks = Vec::new();
        for page in 0..MAX_PAGES {
            let v = self.get(&format!("{path}&page={page}"))?;
            let got = parse_task_list(&v);
            let last = got.is_empty() || v.get("last_page").and_then(Value::as_bool).unwrap_or(true);
            tasks.extend(got);
            if last {
                break;
            }
        }
        Ok(tasks)
    }

    /// Open tasks assigned to the token's user, most recently updated first.
    pub fn my_tasks(&self) -> Result<Vec<Task>> {
        let (team, me) = (self.team_id()?, self.my_user_id()?);
        self.all_tasks(&format!("/team/{team}/task?assignees[]={me}&subtasks=true&order_by=updated"))
    }

    /// Open tasks in one list, subtasks and tasks added from other lists (as sprints do) included.
    pub fn list_tasks(&self, list_id: &str) -> Result<Vec<Task>> {
        self.all_tasks(&format!("/list/{list_id}/task?subtasks=true&include_timl=true&order_by=updated"))
    }

    /// `(id, name)` of the workspace's spaces, limited to `only` when it is not empty.
    pub fn spaces(&self, only: &[String]) -> Result<Vec<(String, String)>> {
        let team = self.team_id()?;
        let v = self.get(&format!("/team/{team}/space?archived=false"))?;
        let spaces = v.get("spaces").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(spaces
            .iter()
            .filter_map(|s| Some((s.get("id")?.as_str()?.to_string(), s.get("name")?.as_str()?.to_string())))
            .filter(|(id, _)| only.is_empty() || only.contains(id))
            .collect())
    }

    /// Every unarchived list in a space, in folders or not.
    pub fn space_lists(&self, space_id: &str, space_name: &str) -> Result<Vec<ListRef>> {
        let folders = self.get(&format!("/space/{space_id}/folder?archived=false"))?;
        let loose = self.get(&format!("/space/{space_id}/list?archived=false"))?;
        Ok(parse_space_lists(space_name, &folders, &loose))
    }

    pub fn task(&self, id: &str) -> Result<Task> {
        parse_task(&self.get(&format!("/task/{id}"))?).ok_or_else(|| anyhow!("task {id} not found"))
    }

    pub fn set_status(&self, id: &str, status: &str) -> Result<()> {
        self.send("PUT", &format!("/task/{id}"), json!({ "status": status }))
    }

    pub fn comment(&self, id: &str, text: &str) -> Result<()> {
        self.send("POST", &format!("/task/{id}/comment"), json!({ "comment_text": text, "notify_all": false }))
    }
}
