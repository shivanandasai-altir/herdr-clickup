/// `herdr agent start --kind` values in herdr 0.9.5; each is also the agent's executable name.
pub const KINDS: &[&str] = &[
    "pi",
    "claude",
    "codex",
    "gemini",
    "cursor",
    "devin",
    "agy",
    "cline",
    "omp",
    "mastracode",
    "opencode",
    "copilot",
    "kimi",
    "kiro",
    "droid",
    "amp",
    "grok",
    "hermes",
    "kilo",
    "qodercli",
    "qwen",
    "letta",
    "maki",
    "muse",
];

const FIRST: [&str; 3] = ["claude", "codex", "pi"];

/// Installed agent kinds to offer: the preferred one, then claude, codex, pi, then the rest in herdr's order.
pub fn agent_choices(installed: impl Fn(&str) -> bool, preferred: Option<&str>) -> Vec<String> {
    let preferred = preferred.filter(|p| KINDS.contains(p));
    let rest = KINDS.iter().filter(|k| !FIRST.contains(k)).copied();
    let mut out: Vec<String> = Vec::new();
    for k in preferred.into_iter().chain(FIRST).chain(rest) {
        if installed(k) && !out.iter().any(|o| o == k) {
            out.push(k.to_string());
        }
    }
    out
}

/// Whether `kind`'s executable is installed. herdr may start plugins without the login PATH, so the usual
/// install dirs are searched too.
pub fn is_installed(kind: &str) -> bool {
    find_executable(kind).is_some()
}

/// Where executable `name` is: PATH, then the usual install dirs.
pub fn find_executable(name: &str) -> Option<std::path::PathBuf> {
    use std::{env, path::PathBuf};
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let extra = [
        ".local/bin",
        ".cargo/bin",
        ".bun/bin",
        ".npm-global/bin",
        ".volta/bin",
    ]
    .map(|d| home.join(d));
    let system = ["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from);
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .chain(extra)
        .chain(system)
        .map(|d| d.join(name))
        .find(|p| is_executable(p))
}

fn is_executable(p: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}
