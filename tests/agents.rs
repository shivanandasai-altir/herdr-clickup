use herdr_clickup::agents::agent_choices;

fn have(names: &'static [&'static str]) -> impl Fn(&str) -> bool {
    move |k| names.contains(&k)
}

#[test]
fn claude_codex_pi_come_first_then_herdrs_order() {
    assert_eq!(agent_choices(have(&["opencode", "pi", "grok", "codex", "claude"]), None), ["claude", "codex", "pi", "opencode", "grok"]);
}

#[test]
fn the_preferred_agent_leads() {
    assert_eq!(agent_choices(have(&["pi", "codex", "claude"]), Some("pi")), ["pi", "claude", "codex"]);
}

#[test]
fn a_preferred_agent_that_is_not_installed_is_ignored() {
    assert_eq!(agent_choices(have(&["codex"]), Some("claude")), ["codex"]);
}

#[test]
fn unknown_kinds_are_never_offered() {
    assert_eq!(agent_choices(have(&["vim", "claude"]), Some("vim")), ["claude"]);
}

#[test]
fn nothing_installed_gives_no_choices() {
    assert!(agent_choices(have(&[]), Some("claude")).is_empty());
}
