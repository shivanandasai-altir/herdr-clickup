use herdr_clickup::task::{parse_task, parse_task_list, Task};
use serde_json::json;

fn sample() -> serde_json::Value {
    // Trimmed from a real GET /task/{id} response.
    json!({
        "id": "86c1abcd",
        "custom_id": null,
        "name": "Fix login redirect",
        "text_content": "After login we land on /404.\nShould go to /home.",
        "description": "After login we land on /404.\nShould go to /home.",
        "status": { "status": "to do", "color": "#d3d3d3", "type": "open" },
        "url": "https://app.clickup.com/t/86c1abcd",
        "assignees": [{ "id": 61481949 }]
    })
}

#[test]
fn parses_a_single_task() {
    assert_eq!(
        parse_task(&sample()),
        Some(Task {
            id: "86c1abcd".into(),
            name: "Fix login redirect".into(),
            status: "to do".into(),
            url: "https://app.clickup.com/t/86c1abcd".into(),
            description: "After login we land on /404.\nShould go to /home.".into(),
        })
    );
}

#[test]
fn missing_description_becomes_empty() {
    let mut v = sample();
    v["text_content"] = json!(null);
    v["description"] = json!(null);
    assert_eq!(parse_task(&v).unwrap().description, "");
}

#[test]
fn rejects_objects_without_id_or_name() {
    assert_eq!(parse_task(&json!({ "name": "no id" })), None);
    assert_eq!(parse_task(&json!({ "id": "x" })), None);
}

#[test]
fn parses_the_team_task_list_response() {
    let list = json!({ "tasks": [sample(), { "id": "bad" }, sample()], "last_page": true });
    let tasks = parse_task_list(&list);
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].name, "Fix login redirect");
}

fn task(id: &str, name: &str, description: &str) -> Task {
    Task {
        id: id.into(),
        name: name.into(),
        status: "to do".into(),
        url: format!("https://app.clickup.com/t/{id}"),
        description: description.into(),
    }
}

mod branch {
    use super::task;
    use herdr_clickup::task::branch_name;

    #[test]
    fn prefixes_the_id_and_slugs_the_name() {
        assert_eq!(branch_name(&task("86c1abcd", "Fix login redirect", "")), "CU-86c1abcd-fix-login-redirect");
    }

    #[test]
    fn collapses_punctuation_and_trims_dashes() {
        assert_eq!(branch_name(&task("abc", "  [API] Add /users endpoint!!  ", "")), "CU-abc-api-add-users-endpoint");
    }

    #[test]
    fn drops_non_ascii_and_caps_slug_at_40_on_a_word_boundary() {
        let b = branch_name(&task("abc", "Überprüfung der sehr langen Aufgabe mit vielen Wörtern und noch mehr Text dazu", ""));
        assert_eq!(b, "CU-abc-berpr-fung-der-sehr-langen-aufgabe-mit");
    }

    #[test]
    fn name_without_letters_is_just_the_id() {
        assert_eq!(branch_name(&task("abc", "???", "")), "CU-abc");
    }
}

mod prompt {
    use super::task;
    use herdr_clickup::task::agent_prompt;

    #[test]
    fn includes_title_url_and_description() {
        assert_eq!(
            agent_prompt(&task("abc", "Fix login", "Users land on /404.")),
            "Work on ClickUp task abc: Fix login\nhttps://app.clickup.com/t/abc\n\nUsers land on /404."
        );
    }

    #[test]
    fn omits_the_description_block_when_empty() {
        assert_eq!(agent_prompt(&task("abc", "Fix login", "  ")), "Work on ClickUp task abc: Fix login\nhttps://app.clickup.com/t/abc");
    }
}
