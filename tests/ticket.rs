use herdr_clickup::task::Task;
use herdr_clickup::ticket::{
    Comment, Ticket, apply_prompt, code_prompt, parse_ticket, spec_prompt, ticket_markdown,
};
use serde_json::json;

// GET /task/{id}?include_subtasks=true, trimmed. Attachments and comments are from real responses;
// checklists and subtasks follow ClickUp's documented shape.
fn task_json() -> serde_json::Value {
    json!({
        "id": "86bb29hug",
        "name": "PO Quantity Adjustment Button",
        "text_content": "Similar to xCRM, the PO should include an Adjust Quantity button.",
        "status": { "status": "ready for eng review" },
        "url": "https://app.clickup.com/t/86bb29hug",
        "assignees": [{ "id": 1, "username": "Shivananda Sai" }],
        "tags": [{ "name": "purchasing" }, { "name": "ui" }],
        "parent": "86parent",
        "custom_fields": [
            { "id": "cf1", "name": "Email", "type": "short_text", "value": "ops@example.com", "type_config": {} },
            { "id": "cf2", "name": "Priority", "type": "drop_down", "value": "p1", "type_config": { "options": [{ "id": "p1", "name": "High" }] } },
            { "id": "cf3", "name": "Impacted users", "type": "labels", "value": ["u1", "u2"], "type_config": { "options": [{ "id": "u1", "name": "buyers" }, { "id": "u2", "name": "admins" }] } },
            { "id": "cf4", "name": "Owner", "type": "users", "value": 7, "type_config": {} },
            { "id": "cf5", "name": "Empty", "type": "short_text", "value": null, "type_config": {} }
        ],
        "checklists": [{ "name": "Acceptance", "items": [
            { "name": "Button reduces ordered qty", "resolved": true },
            { "name": "Remaining balance is closed", "resolved": false }
        ]}],
        "subtasks": [{ "id": "86s1", "name": "API: adjust endpoint", "status": { "status": "to do" } }],
        "attachments": [{ "title": "mock.png", "url": "https://t8651091.p.clickup-attachments.com/mock.png" }]
    })
}

fn comments_json() -> serde_json::Value {
    json!({ "comments": [
        { "user": { "username": "Deepti Vaidyula" }, "date": "1790839457019", "comment_text": "Retry 3 times.\nThanks" }
    ]})
}

#[test]
fn parses_the_whole_ticket() {
    let t = parse_ticket(&task_json(), &comments_json()).unwrap();
    assert_eq!(t.task.id, "86bb29hug");
    assert_eq!(t.tags, ["purchasing", "ui"]);
    assert_eq!(t.parent.as_deref(), Some("86parent"));
    assert_eq!(
        t.custom_fields,
        [
            ("Email".to_string(), "ops@example.com".to_string()),
            ("Priority".to_string(), "High".to_string()),
            ("Impacted users".to_string(), "buyers, admins".to_string())
        ]
    );
    assert_eq!(
        t.checklists,
        [(
            "Acceptance".to_string(),
            vec![
                ("Button reduces ordered qty".to_string(), true),
                ("Remaining balance is closed".to_string(), false)
            ]
        )]
    );
    assert_eq!(
        t.subtasks,
        [(
            "86s1".to_string(),
            "API: adjust endpoint".to_string(),
            "to do".to_string()
        )]
    );
    assert_eq!(
        t.attachments,
        [(
            "mock.png".to_string(),
            "https://t8651091.p.clickup-attachments.com/mock.png".to_string()
        )]
    );
    assert_eq!(
        t.comments,
        [Comment {
            user: "Deepti Vaidyula".into(),
            date_ms: 1790839457019,
            text: "Retry 3 times.\nThanks".into()
        }]
    );
}

#[test]
fn a_bare_task_parses_with_empty_extras() {
    let t = parse_ticket(&json!({ "id": "x", "name": "task" }), &json!({})).unwrap();
    assert!(
        t.tags.is_empty()
            && t.parent.is_none()
            && t.custom_fields.is_empty()
            && t.checklists.is_empty()
            && t.subtasks.is_empty()
            && t.comments.is_empty()
    );
    assert_eq!(parse_ticket(&json!({ "name": "no id" }), &json!({})), None);
}

#[test]
fn renders_the_ticket_as_markdown() {
    let t = parse_ticket(&task_json(), &comments_json()).unwrap();
    assert_eq!(
        ticket_markdown(&t),
        "# PO Quantity Adjustment Button

- ClickUp: https://app.clickup.com/t/86bb29hug
- Status: ready for eng review
- Assignees: Shivananda Sai
- Tags: purchasing, ui
- Parent task: 86parent

## Custom fields

- Email: ops@example.com
- Priority: High
- Impacted users: buyers, admins

## Description

Similar to xCRM, the PO should include an Adjust Quantity button.

## Checklist: Acceptance

- [x] Button reduces ordered qty
- [ ] Remaining balance is closed

## Subtasks

- API: adjust endpoint (to do) — 86s1

## Attachments

- [mock.png](https://t8651091.p.clickup-attachments.com/mock.png)

## Comments

**Deepti Vaidyula** · 2026-10-01

> Retry 3 times.
> Thanks
"
    );
}

#[test]
fn a_bare_ticket_says_it_has_no_description() {
    let t = parse_ticket(
        &json!({ "id": "x", "name": "task", "url": "u" }),
        &json!({}),
    )
    .unwrap();
    assert_eq!(
        ticket_markdown(&t),
        "# task\n\n- ClickUp: u\n\n## Description\n\n_No description._\n"
    );
}

fn task() -> Task {
    parse_ticket(&task_json(), &json!({})).unwrap().task
}

#[test]
fn the_spec_prompt_asks_for_artifacts_and_forbids_code() {
    assert_eq!(
        spec_prompt(&task(), "cu-86bb29hug-po-quantity", "openspec/changes/cu-86bb29hug-po-quantity/clickup.md"),
        "Plan ClickUp task 86bb29hug (PO Quantity Adjustment Button) as the OpenSpec change `cu-86bb29hug-po-quantity`: spec first, no code yet.

The full ticket is in openspec/changes/cu-86bb29hug-po-quantity/clickup.md.

1. Write the change's artifacts in order (proposal, specs, design, tasks), each following `openspec instructions <artifact> --change cu-86bb29hug-po-quantity`.
2. Run `openspec validate cu-86bb29hug-po-quantity` and fix what it reports.
3. Do not write or change any application code. Stop when the change validates, and list the open questions and assumptions for me to review."
    );
}

#[test]
fn the_code_prompt_carries_the_whole_ticket() {
    let t: Ticket = parse_ticket(&task_json(), &comments_json()).unwrap();
    let p = code_prompt(&t);
    assert!(
        p.starts_with(
            "Work on ClickUp task 86bb29hug. The full ticket:\n\n# PO Quantity Adjustment Button\n"
        ),
        "{p}"
    );
    assert!(
        p.contains("- [ ] Remaining balance is closed") && p.contains("> Retry 3 times."),
        "{p}"
    );
}

#[test]
fn the_apply_prompt_sends_the_change_workflow() {
    let p = apply_prompt("cu-86a-x");
    assert!(
        p.contains("openspec instructions apply --change cu-86a-x"),
        "{p}"
    );
    assert!(p.contains("tests"), "{p}");
}
