//! Typed projections follow scalar style changes and comment relocation.

use astral_yaml::comments::CommentPlacement;
use astral_yaml_serde::{Editable, edit::CollectionStyle};
use serde_json::json;

#[test]
fn nested_layout_changes_keep_the_typed_model_current() {
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "outer:\n  value: &a old\n",
            "outer:\n  value: &a old\n\n# root\n",
            "outer:\n  value: &a old\ncopy: *a\n",
        ] {
            let source = source.replace('\n', newline);
            for seed in 1..=128u64 {
                let mut document = Editable::<serde_json::Value>::parse(&source).unwrap();
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
                        .edit(|editor| {
                            match operation {
                                0 => {
                                    editor
                                        .root_mut()
                                        .field("outer")?
                                        .field("value")?
                                        .set("null")?;
                                }
                                1 => {
                                    let syntax = editor.syntax_mut();
                                    let outer =
                                        syntax.mapping_entries(&syntax.root())?[0].1.clone();
                                    let value = syntax.mapping_entries(&outer)?[0].1.clone();
                                    syntax.replace(&value, "&a |+\n  new\n\n")?;
                                }
                                2 | 3 => editor.root_mut().field("outer")?.set_collection_style(
                                    if operation == 2 {
                                        CollectionStyle::Flow
                                    } else {
                                        CollectionStyle::Block
                                    },
                                )?,
                                _ => {
                                    let owner = usize::from(operation == 4 || operation == 5);
                                    let mut cursor = editor.root_mut();
                                    if owner == 1 {
                                        cursor = cursor.field("outer")?;
                                    }
                                    if operation % 2 == 0 {
                                        cursor
                                            .append_comment(" added", CommentPlacement::Detached)?;
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
                        let value = if operation == 0 { "null" } else { "new\n\n" };
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
    }
}

fn check_comments(document: &Editable<serde_json::Value>, comments: &[Vec<&str>; 2]) {
    let syntax = document.syntax();
    let root = syntax.root();
    let astral_yaml::edit::NodeKind::Mapping { entries, .. } = syntax.node(root).unwrap().kind()
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
