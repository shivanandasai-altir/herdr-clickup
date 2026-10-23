/// The task id in `app.clickup.com/t/<id>` or `app.clickup.com/t/<team>/<id>`.
pub fn task_id_from_url(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://app.clickup.com/t/")?;
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    let id = path.split('/').rfind(|s| !s.is_empty())?;
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
        .then(|| id.to_string())
}
