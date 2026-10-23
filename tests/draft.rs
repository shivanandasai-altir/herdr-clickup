use herdr_clickup::app::{edit_text, parse_edit};
use herdr_clickup::hook::Update;

fn update(comment: &str, status: Option<&str>) -> Update {
    Update {
        comment: comment.into(),
        status: status.map(str::to_string),
    }
}

#[test]
fn a_comment_and_status_round_trip_through_the_editor() {
    let u = update(
        "Fixed the PO button.\n\n- new endpoint\n- new test",
        Some("review"),
    );
    assert_eq!(parse_edit(&edit_text(&u)), u);
}

#[test]
fn the_editor_can_change_the_comment_and_clear_the_status() {
    let text =
        "# herdr-clickup draft: edit\n\nNew wording with a `status: review` note.\n\nstatus:\n";
    assert_eq!(
        parse_edit(text),
        update("New wording with a `status: review` note.", None)
    );
}

#[test]
fn a_status_only_draft_keeps_an_empty_comment() {
    let u = update("", Some("in progress"));
    assert_eq!(parse_edit(&edit_text(&u)), u);
}

#[test]
fn a_comment_with_markdown_headings_survives() {
    let text = "# herdr-clickup draft: edit\n\n# What changed\n\n- the button\n\nstatus: review\n";
    assert_eq!(
        parse_edit(text),
        update("# What changed\n\n- the button", Some("review"))
    );
}

#[test]
fn a_missing_status_line_means_no_status_change() {
    assert_eq!(parse_edit("just a comment"), update("just a comment", None));
    assert_eq!(parse_edit(""), update("", None));
}

#[test]
fn the_edit_file_names_the_status_and_the_comment() {
    let text = edit_text(&update("Done.", Some("review")));
    assert!(text.contains("status: review"), "{text}");
    assert!(text.contains("Done."), "{text}");
    assert!(text.starts_with("# herdr-clickup draft:"), "{text}");
}
