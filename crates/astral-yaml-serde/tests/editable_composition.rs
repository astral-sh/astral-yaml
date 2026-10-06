//! Typed transactions checked against ordinary Rust values, not emitter comparisons.

use astral_yaml_serde::{
    Editable, Error,
    edit::{CollectionStyle, Editor, NodeKind},
};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
enum Operation {
    String,
    Boolean,
    Integer,
    Null,
    PushString,
    PushObject,
    InsertFirst,
    RemoveFirst,
    SetEntry,
    RemoveEntry,
    EditSelectedEntry,
    Flow,
    Block,
    Multiline,
    Mapping,
    Sequence,
}

impl Operation {
    fn apply(self, editor: &mut Editor<'_, '_>) -> Result<(), Error> {
        match self {
            Self::Multiline => editor.root_mut().field("value")?.set("first\n\nlast\n\n")?,
            Self::Mapping => editor
                .root_mut()
                .field("value")?
                .set(json!({"nested": ["false", null]}))?,
            Self::Sequence => editor.root_mut().field("value")?.set([true, false])?,
            Self::Block => editor
                .root_mut()
                .set_collection_style(CollectionStyle::Block)?,
            Self::String => editor.root_mut().field("value")?.set("null")?,
            Self::Boolean => editor.root_mut().field("value")?.set(false)?,
            Self::Integer => editor.root_mut().field("value")?.set(i64::MIN)?,
            Self::Null => editor.root_mut().field("value")?.set(())?,
            Self::PushString => {
                editor.root_mut().field("items")?.push("true")?;
            }
            Self::PushObject => {
                editor
                    .root_mut()
                    .field("items")?
                    .push(json!({"inner": [true, null, "false"]}))?;
            }
            Self::InsertFirst => {
                editor.root_mut().field("items")?.insert(0, "0123")?;
            }
            Self::RemoveFirst => {
                let mut items = editor.root_mut().field("items")?;
                if matches!(items.node()?.kind(), NodeKind::Sequence { items, .. } if !items.is_empty())
                {
                    items.remove_item(0)?;
                }
            }
            Self::SetEntry => {
                editor.root_mut().field("map")?.set_entry("true", false)?;
            }
            Self::RemoveEntry => {
                editor.root_mut().field("map")?.remove_field("true")?;
            }
            Self::EditSelectedEntry => editor
                .root_mut()
                .field("map")?
                .set_entry("true", "first")?
                .set(42)?,
            Self::Flow => editor
                .root_mut()
                .set_collection_style(CollectionStyle::Flow)?,
        }
        Ok(())
    }

    fn model(self, value: &mut Value) {
        match self {
            Self::Multiline => value["value"] = json!("first\n\nlast\n\n"),
            Self::Mapping => value["value"] = json!({"nested": ["false", null]}),
            Self::Sequence => value["value"] = json!([true, false]),
            Self::String => value["value"] = json!("null"),
            Self::Boolean => value["value"] = json!(false),
            Self::Integer => value["value"] = json!(i64::MIN),
            Self::Null => value["value"] = Value::Null,
            Self::PushString => value["items"].as_array_mut().unwrap().push(json!("true")),
            Self::PushObject => value["items"]
                .as_array_mut()
                .unwrap()
                .push(json!({"inner": [true, null, "false"]})),
            Self::InsertFirst => value["items"]
                .as_array_mut()
                .unwrap()
                .insert(0, json!("0123")),
            Self::RemoveFirst => {
                let items = value["items"].as_array_mut().unwrap();
                if !items.is_empty() {
                    items.remove(0);
                }
            }
            Self::SetEntry => value["map"]["true"] = json!(false),
            Self::RemoveEntry => {
                value["map"].as_object_mut().unwrap().remove("true");
            }
            Self::EditSelectedEntry => value["map"]["true"] = json!(42),
            Self::Flow | Self::Block => {}
        }
        // The alias continues to reference this occurrence as its value changes.
        value["copy"] = value["value"].clone();
    }
}

#[test]
fn typed_edits_match_an_independent_value_model() {
    use Operation::{
        Boolean, EditSelectedEntry, Flow, InsertFirst, Integer, Null, PushObject, PushString,
        RemoveEntry, RemoveFirst, SetEntry, String,
    };
    let operations = [
        String,
        Boolean,
        Integer,
        Null,
        PushString,
        PushObject,
        InsertFirst,
        RemoveFirst,
        SetEntry,
        RemoveEntry,
        EditSelectedEntry,
        Flow,
    ];
    for newline in ["\n", "\r\n", "\r"] {
        for &source in SOURCES {
            let source = source.replace('\n', newline);
            for first in operations {
                for second in operations {
                    for third in operations {
                        let sequence = [first, second, third];
                        check_sequence(&source, sequence);
                    }
                }
            }
        }
    }
}

const SOURCES: &[&str] = &[
    "value: &a old # value\ncopy: *a\nitems: [x]\nmap: {key: value}\n",
    "{value: &a 'old', copy: *a, items: [x], map: {key: value}} # root\n",
    "--- # start\nvalue: &a |-\n  old\ncopy: *a\nitems:\n- x\nmap:\n  key: value\n... # end\n",
    "? value\n: &a old\n? copy\n: *a\n? items\n: [x]\n? map\n: {key: value}\n",
];

#[test]
fn longer_typed_edits_match_the_independent_model() {
    use Operation::*;
    let operations = [
        String,
        Boolean,
        Integer,
        Null,
        PushString,
        PushObject,
        InsertFirst,
        RemoveFirst,
        SetEntry,
        RemoveEntry,
        EditSelectedEntry,
        Flow,
        Block,
        Multiline,
        Mapping,
        Sequence,
    ];
    for newline in ["\n", "\r\n", "\r"] {
        for &source in SOURCES {
            let source = source.replace('\n', newline);
            for seed in 1..=128u64 {
                let mut state = seed;
                let mut sequence = [String; 64];
                for operation in &mut sequence {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    let index = state % u64::try_from(operations.len()).unwrap();
                    *operation = operations[usize::try_from(index).unwrap()];
                }
                check_sequence(&source, sequence);
            }
        }
    }
}

fn check_sequence<const N: usize>(source: &str, sequence: [Operation; N]) {
    let initial = json!({"value": "old", "copy": "old", "items": ["x"], "map": {"key": "value"}});
    let mut expected = initial.clone();
    for operation in sequence {
        operation.model(&mut expected);
    }
    let mut combined = Editable::<Value>::parse(source).unwrap();
    combined
        .edit(|editor| {
            for operation in sequence {
                operation.apply(editor)?;
            }
            Ok(())
        })
        .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}, combined: {error}"));
    assert_eq!(combined.value(), &expected, "{source:?}, {sequence:?}");
    check_alias(&combined);

    let mut separate = Editable::<Value>::parse(source).unwrap();
    let mut step = initial;
    for operation in sequence {
        operation.model(&mut step);
        separate
            .edit(|editor| operation.apply(editor))
            .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}, separate: {error}"));
        assert_eq!(separate.value(), &step, "{source:?}, {sequence:?}");
        check_alias(&separate);
    }
}

fn check_alias(document: &Editable<Value>) {
    let syntax = document.syntax();
    let NodeKind::Mapping { entries, .. } = syntax.node(syntax.root()).unwrap().kind() else {
        unreachable!()
    };
    let field = |name| {
        entries.iter().find_map(|&(key, value)| {
        matches!(syntax.node(key).unwrap().kind(), NodeKind::Scalar { value, .. } if value == name).then_some(value)
    }).unwrap()
    };
    let value = field("value");
    assert_eq!(syntax.node(value).unwrap().anchor(), Some("a"));
    assert!(
        matches!(syntax.node(field("copy")).unwrap().kind(), NodeKind::Alias { target, .. } if *target == value)
    );
}

#[test]
fn typed_root_replacements_preserve_values_and_comment_attachments() {
    let values = [
        json!(null),
        json!(true),
        json!(42),
        json!(1.5),
        json!(""),
        json!("null"),
        json!("0123"),
        json!("---"),
        json!("a: b"),
        json!("\r\n"),
        json!("line\nline\n"),
        json!("\u{feff}"),
        json!([true, "false"]),
        json!({"key": null}),
    ];
    let sources = [
        "old # inline\n",
        "[ # open\n  old\n] # close\n",
        "&a # property\n[ # open\n  old\n] # close\n",
    ];
    for newline in ["\n", "\r\n", "\r"] {
        for source in sources {
            let source = source.replace('\n', newline);
            for value in &values {
                let mut document = Editable::<Value>::parse(&source).unwrap();
                let comments = |document: &Editable<Value>| {
                    document
                        .syntax()
                        .node(document.syntax().root())
                        .unwrap()
                        .comments()
                        .iter()
                        .map(|comment| (comment.text().to_owned(), comment.placement()))
                        .collect::<Vec<_>>()
                };
                let expected_comments = comments(&document);
                document
                    .edit(|editor| editor.root_mut().set(value))
                    .unwrap_or_else(|error| panic!("{source:?}, {value:?}: {error}"));
                assert_eq!(document.value(), value, "{source:?}");
                assert_eq!(comments(&document), expected_comments, "{source:?}");
            }
        }
    }
}

#[test]
fn typed_comment_insertions_share_the_root_header_plan() {
    use astral_yaml_dev::edit_diff;
    use astral_yaml_serde::edit::CommentPlacement;

    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "old # first\n".replace('\n', newline);
        let mut document = Editable::<Value>::parse(&source).unwrap();
        document
            .edit(|editor| {
                let mut root = editor.root_mut();
                root.append_comment(" second", CommentPlacement::Inline)?;
                root.append_comment(" third", CommentPlacement::Inline)?;
                root.set(["new"])
            })
            .unwrap();
        assert_eq!(document.value(), &json!(["new"]));
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # first
    +--- # first
    +[ # second
    +  "new"] # third

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # first␍
    +--- # first␍
    +[ # second␍
    +  "new"] # third␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # first␍+--- # first␍+[ # second␍+  "new"] # third␍
    "#);
}

#[test]
fn typed_failure_discards_generated_framing() {
    use astral_yaml_serde::edit::CommentPlacement;

    let source = "old # first\n";
    let mut document = Editable::<String>::parse(source).unwrap();
    let error = document
        .edit(|editor| {
            let mut root = editor.root_mut();
            root.append_comment(" second", CommentPlacement::Inline)?;
            root.append_comment(" third", CommentPlacement::Inline)?;
            root.set(["new"])
        })
        .unwrap_err();
    assert_eq!(document.source(), source);
    assert_eq!(document.value(), "old");
    insta::assert_snapshot!(error.message, @"expected a scalar");
}
