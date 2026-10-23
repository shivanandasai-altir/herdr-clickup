use herdr_clickup::repos::repo_choices;

fn found() -> Vec<String> {
    ["/w/chip1-webui", "/w/chip1-api-hub", "/w/chip1-webui", "/w/zalio"].map(String::from).to_vec()
}

#[test]
fn found_repos_in_workspace_order_without_duplicates() {
    assert_eq!(repo_choices(&found(), None, None), ["/w/chip1-webui", "/w/chip1-api-hub", "/w/zalio"]);
}

#[test]
fn the_current_workspace_repo_comes_first_then_the_last_pick() {
    assert_eq!(
        repo_choices(&found(), Some("/w/zalio"), Some("/w/chip1-api-hub")),
        ["/w/zalio", "/w/chip1-api-hub", "/w/chip1-webui"]
    );
}

#[test]
fn the_last_pick_leads_when_the_current_workspace_is_not_a_repo() {
    assert_eq!(repo_choices(&found(), None, Some("/w/zalio")), ["/w/zalio", "/w/chip1-webui", "/w/chip1-api-hub"]);
}

#[test]
fn a_last_pick_no_longer_open_is_still_offered() {
    assert_eq!(repo_choices(&[], None, Some("/w/old")), ["/w/old"]);
}

#[test]
fn empty_paths_are_ignored() {
    assert!(repo_choices(&["".into()], Some(""), Some("")).is_empty());
}
