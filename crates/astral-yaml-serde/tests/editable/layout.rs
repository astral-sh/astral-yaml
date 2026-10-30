//! Typed projections follow scalar style changes and comment relocation.

use test_case::test_case;

use astral_yaml::comments::CommentPlacement;
use astral_yaml_dev::indoc;
use astral_yaml_serde::Editable;
use astral_yaml_serde::edit::CollectionStyle;
use serde_json::json;

#[test_case(
    indoc! {r"
        outer:
          value: &a old
    "};
    "nested value"
)]
#[test_case(
    indoc! {r"
        outer:
          value: &a old

        # root
    "};
    "root comment"
)]
#[test_case(
    indoc! {r"
        outer:
          value: &a old
        copy: *a
    "};
    "external alias"
)]
fn nested_layout_changes_keep_the_typed_model_current(source: &str) {
    for seed in 1..=128u64 {
        let mut document = Editable::<serde_json::Value>::parse(source).unwrap();
        let mut expected = json!({"outer": {"value": "old"}});
        if source.contains("copy:") {
            expected["copy"] = json!("old");
        }
        let mut comments = [Vec::new(), Vec::new()];
        if source.contains("# root") {
            comments[0].push(" root");
        }
        let mut state = seed;
        for _ in 0..32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let operation = state % 8;
            document
                .edit(|root| {
                    match operation {
                        0 => {
                            root.field("outer")?.field("value")?.set("null")?;
                        }
                        1 => {
                            root.at(["outer", "value"])?.set("new\n\n")?;
                        }
                        2 | 3 => root
                            .field("outer")?
                            .set_collection_style(if operation == 2 {
                                CollectionStyle::Flow
                            } else {
                                CollectionStyle::Block
                            })?,
                        _ => {
                            let owner = usize::from(operation == 4 || operation == 5);
                            let mut cursor = root;
                            if owner == 1 {
                                cursor = cursor.field("outer")?;
                            }
                            if operation % 2 == 0 {
                                cursor.append_comment(" added", CommentPlacement::Detached)?;
                                comments[owner].push(" added");
                            } else if !comments[owner].is_empty() {
                                cursor.remove_comment(0)?;
                                comments[owner].remove(0);
                            }
                        }
                    }
                    Ok(())
                })
                .unwrap_or_else(|error| {
                    panic!("{source:?}, seed {seed}, operation {operation}: {error}")
                });
            if operation <= 1 {
                let value = if operation == 0 {
                    "null"
                } else {
                    indoc! {r"
                        new

                    "}
                };
                expected["outer"]["value"] = json!(value);
                if source.contains("copy:") {
                    expected["copy"] = json!(value);
                }
            }
            assert_eq!(document.value(), &expected);
            check_comments(&document, &comments);
        }
    }
}

fn check_comments(document: &Editable<serde_json::Value>, comments: &[Vec<&str>; 2]) {
    let syntax = document.syntax();
    let root = syntax.root();
    let astral_yaml_serde::edit::NodeKind::Mapping { entries, .. } =
        syntax.node(root).unwrap().kind()
    else {
        panic!("expected mapping")
    };
    for (owner, expected) in [root, entries[0].1].into_iter().zip(comments) {
        let actual: Vec<_> = syntax
            .node(owner)
            .unwrap()
            .comments()
            .iter()
            .map(|comment| (comment.text(), comment.placement()))
            .collect();
        assert_eq!(
            actual,
            expected
                .iter()
                .map(|&text| (text, CommentPlacement::Detached))
                .collect::<Vec<_>>()
        );
    }
}
