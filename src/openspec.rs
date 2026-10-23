//! The OpenSpec CLI, run inside a task's worktree.
use crate::agents::find_executable;
use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use std::{path::Path, path::PathBuf, process::Command};

pub const NOT_INSTALLED: &str =
    "OpenSpec is not installed; install it with `npm i -g @fission-ai/openspec` (or bun)";

/// `openspec init --tools` name for a herdr agent kind, where OpenSpec supports it.
pub fn tool_for(agent: &str) -> Option<&'static str> {
    Some(match agent {
        "claude" => "claude",
        "codex" => "codex",
        "pi" => "pi",
        "omp" => "oh-my-pi",
        "cursor" => "cursor",
        "opencode" => "opencode",
        "gemini" => "gemini",
        "grok" => "grok",
        "hermes" => "hermes",
        "amp" => "amp",
        "cline" => "cline",
        "kimi" => "kimi",
        "kiro" => "kiro",
        "qwen" => "qwen",
        "droid" => "factory",
        "copilot" => "github-copilot",
        "kilo" => "kilocode",
        "qodercli" => "qoder",
        "devin" => "devin",
        _ => return None,
    })
}

fn bin() -> Result<PathBuf> {
    find_executable("openspec").ok_or_else(|| anyhow!(NOT_INSTALLED))
}

pub fn is_installed() -> bool {
    bin().is_ok()
}

fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new(bin()?)
        .args(args)
        .current_dir(dir)
        .output()
        .context("running openspec")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let msg = if err.trim().is_empty() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            err.to_string()
        };
        bail!("openspec {}: {}", args.join(" "), msg.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Sets OpenSpec up in `dir` (with `agent`'s commands) unless it already has it. True when it did.
pub fn ensure_init(dir: &Path, agent: &str) -> Result<bool> {
    if dir.join("openspec").is_dir() {
        return Ok(false);
    }
    run(
        dir,
        &[
            "init",
            "--tools",
            tool_for(agent).unwrap_or("agents"),
            "--no-animation",
            ".",
        ],
    )?;
    Ok(true)
}

/// Fails unless the change in `dir` passes strict validation.
pub fn validate(dir: &Path, change: &str) -> Result<()> {
    run(dir, &["validate", change, "--strict", "--no-interactive"])?;
    Ok(())
}

/// Creates the change `name` in `dir` and returns its folder.
pub fn new_change(dir: &Path, name: &str, description: &str) -> Result<PathBuf> {
    let out = run(
        dir,
        &[
            "new",
            "change",
            name,
            "--description",
            description,
            "--json",
        ],
    )?;
    let v: Value = serde_json::from_str(&out)
        .with_context(|| format!("openspec new change: bad JSON: {out}"))?;
    v.pointer("/change/path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("openspec new change: no path in {out}"))
}
