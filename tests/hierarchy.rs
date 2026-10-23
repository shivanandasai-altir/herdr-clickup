use herdr_clickup::hierarchy::{parse_space_lists, ListRef};
use serde_json::json;

// Trimmed from real responses: ClickUp sends dates as strings of ms.
fn folders() -> serde_json::Value {
    json!({ "folders": [
        { "id": "f1", "name": "Weekly Sprints", "lists": [
            { "id": "l1", "name": "Sprint 17 (10/20 - 10/26)", "start_date": "1792434600000", "due_date": "1793039399999", "task_count": 51 }
        ]},
        { "id": "f2", "name": "Internal Defects", "lists": [
            { "id": "l2", "name": "Bugs", "start_date": null, "due_date": null, "task_count": 202 },
            { "name": "no id" }
        ]}
    ]})
}

#[test]
fn flattens_folder_and_folderless_lists() {
    let folderless = json!({ "lists": [{ "id": "l3", "name": "Inbox" }] });
    let lists = parse_space_lists("Chip1", &folders(), &folderless);
    assert_eq!(
        lists,
        vec![
            ListRef {
                id: "l1".into(),
                name: "Sprint 17 (10/20 - 10/26)".into(),
                space: "Chip1".into(),
                folder: Some("Weekly Sprints".into()),
                start: Some(1792434600000),
                due: Some(1793039399999),
                task_count: Some(51),
            },
            ListRef {
                id: "l2".into(),
                name: "Bugs".into(),
                space: "Chip1".into(),
                folder: Some("Internal Defects".into()),
                start: None,
                due: None,
                task_count: Some(202),
            },
            ListRef {
                id: "l3".into(),
                name: "Inbox".into(),
                space: "Chip1".into(),
                folder: None,
                start: None,
                due: None,
                task_count: None,
            },
        ]
    );
}

#[test]
fn accepts_numeric_dates_and_counts_as_strings() {
    let f = json!({ "folders": [{ "name": "S", "lists": [
        { "id": "l", "name": "Sprint", "start_date": 5, "due_date": "9", "task_count": "3" }
    ]}]});
    let l = &parse_space_lists("X", &f, &json!({}))[0];
    assert_eq!((l.start, l.due, l.task_count), (Some(5), Some(9), Some(3)));
}

#[test]
fn empty_responses_give_no_lists() {
    assert!(parse_space_lists("X", &json!({}), &json!(null)).is_empty());
}

mod sprints {
    use herdr_clickup::hierarchy::{current_sprints, ListRef};

    fn list(id: &str, start: Option<i64>, due: Option<i64>) -> ListRef {
        ListRef { id: id.into(), name: id.into(), space: "S".into(), folder: None, start, due, task_count: None }
    }

    fn ids(v: Vec<&ListRef>) -> Vec<&str> {
        v.into_iter().map(|l| l.id.as_str()).collect()
    }

    #[test]
    fn picks_lists_whose_dates_contain_now_inclusive() {
        let lists = [
            list("past", Some(0), Some(99)),
            list("starts-now", Some(100), Some(200)),
            list("ends-now", Some(50), Some(100)),
            list("future", Some(101), Some(300)),
            list("not-a-sprint", None, None),
        ];
        assert_eq!(ids(current_sprints(&lists, 100)), ["starts-now", "ends-now"]);
    }

    #[test]
    fn a_list_needs_both_dates_to_be_a_sprint() {
        let lists = [list("only-start", Some(0), None), list("only-due", None, Some(1000))];
        assert!(current_sprints(&lists, 10).is_empty());
    }
}
