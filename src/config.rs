use anyhow::{Result, bail};
use std::{collections::HashMap, env, fs, path::PathBuf};

/// Settings from `$HERDR_PLUGIN_CONFIG_DIR/.env`; real environment variables win.
#[derive(Clone)]
pub struct Config {
    pub token: String,
    pub team_id: Option<String>,
    /// Spaces the board shows; empty means all.
    pub space_ids: Vec<String>,
    /// The agent offered first when none has been picked yet.
    pub agent: Option<String>,
    pub status_started: Option<String>,
    pub status_done: Option<String>,
    /// Set when a spec-first start begins and when its spec is drafted; off by default.
    pub status_spec: Option<String>,
}

fn config_dir() -> Option<PathBuf> {
    env::var_os("HERDR_PLUGIN_CONFIG_DIR").map(PathBuf::from)
}

fn read_env_file() -> HashMap<String, String> {
    let Some(text) = config_dir().and_then(|d| fs::read_to_string(d.join(".env")).ok()) else {
        return HashMap::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        .collect()
}

impl Config {
    pub fn load() -> Result<Config> {
        let file = read_env_file();
        let get = |k: &str| {
            env::var(k)
                .ok()
                .or_else(|| file.get(k).cloned())
                .filter(|v| !v.is_empty())
        };
        let Some(token) = get("CLICKUP_TOKEN") else {
            let dir = config_dir()
                .map(|d| d.display().to_string())
                .unwrap_or("the plugin config dir".into());
            bail!("CLICKUP_TOKEN is not set; add it to {dir}/.env");
        };
        // An explicit empty value turns a status change off.
        let status =
            |k: &str, default: &str| match env::var(k).ok().or_else(|| file.get(k).cloned()) {
                Some(v) if v.is_empty() => None,
                Some(v) => Some(v),
                None => Some(default.to_string()),
            };
        Ok(Config {
            token,
            team_id: get("CLICKUP_TEAM_ID"),
            space_ids: get("CLICKUP_SPACE_IDS")
                .map(|v| {
                    v.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            agent: get("CLICKUP_AGENT"),
            status_started: status("CLICKUP_STATUS_STARTED", "in progress"),
            status_done: status("CLICKUP_STATUS_DONE", "review"),
            status_spec: get("CLICKUP_STATUS_SPEC"),
        })
    }
}
