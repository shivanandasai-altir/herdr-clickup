use crate::hook::State;
use anyhow::Result;
use std::{env, fs, path::PathBuf};

fn path() -> PathBuf {
    let dir = env::var_os("HERDR_PLUGIN_STATE_DIR").map(PathBuf::from).unwrap_or_else(env::temp_dir);
    dir.join("panes.json")
}

pub fn load() -> State {
    fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Writes via a temp file so a concurrent hook never reads half a file.
pub fn save(state: &State) -> Result<()> {
    let p = path();
    let tmp = p.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    fs::rename(tmp, p)?;
    Ok(())
}
