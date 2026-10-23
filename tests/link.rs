use herdr_clickup::link::task_id_from_url;

#[test]
fn plain_task_url() {
    assert_eq!(task_id_from_url("https://app.clickup.com/t/86c1abcd").as_deref(), Some("86c1abcd"));
}

#[test]
fn team_scoped_task_url() {
    assert_eq!(task_id_from_url("https://app.clickup.com/t/9012345678/86c1abcd").as_deref(), Some("86c1abcd"));
}

#[test]
fn ignores_query_fragment_and_trailing_slash() {
    assert_eq!(task_id_from_url("https://app.clickup.com/t/86c1abcd/?comment=1#x").as_deref(), Some("86c1abcd"));
}

#[test]
fn rejects_other_urls() {
    assert_eq!(task_id_from_url("https://app.clickup.com/9012345/v/li/123"), None);
    assert_eq!(task_id_from_url("https://example.com/t/86c1abcd"), None);
    assert_eq!(task_id_from_url("https://app.clickup.com/t/"), None);
}
