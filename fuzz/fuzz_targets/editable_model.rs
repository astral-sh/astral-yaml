#![no_main]

//! Valid operations must succeed and match an independent model after each batch.

use std::collections::BTreeMap;

use astral_yaml::comments::CommentPlacement;
use astral_yaml_dev::formatdoc;
use astral_yaml_serde::{
    Editable, Error,
    edit::{CollectionStyle, Cursor, NodeKind, ScalarStyle},
};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

const OPERATIONS: u8 = 23;

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    value: String,
    copy: String,
    items: Vec<String>,
    map: BTreeMap<String, String>,
    nested: Nested,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum Nested {
    Mapping { value: String },
    Sequence(Vec<Self>),
}

impl Nested {
    fn value_mut(&mut self) -> &mut String {
        match self {
            Self::Mapping { value } => value,
            Self::Sequence(items) => items[0].value_mut(),
        }
    }
}

fn nested<'editor, 'doc>(root: Cursor<'editor, 'doc>) -> Result<Cursor<'editor, 'doc>, Error> {
    let mut cursor = root.field("nested")?;
    while matches!(cursor.node()?.kind(), NodeKind::Sequence { .. }) {
        cursor = cursor.item(0)?;
    }
    Ok(cursor)
}

// A previous flow-root edit can make every enclosing sequence flow-style.
// Explicit block requests must reopen the entire path to the nested mapping.
fn set_nested_style(root: Cursor<'_, '_>, style: CollectionStyle) -> Result<(), Error> {
    let mut cursor = root.field("nested")?;
    loop {
        cursor.set_collection_style(style)?;
        if !matches!(cursor.node()?.kind(), NodeKind::Sequence { .. }) {
            return Ok(());
        }
        cursor = cursor.item(0)?;
    }
}

#[derive(Default)]
struct Comments {
    root: Vec<(String, CommentPlacement)>,
    nested: Vec<(String, CommentPlacement)>,
}

fn apply(
    mut root: Cursor<'_, '_>,
    expected: &mut Config,
    comments: &mut Comments,
    operation: &[u8; 2],
    text: &str,
) -> Result<(), Error> {
    let selector = usize::from(operation[1]);
    let key = ["null", "true", "01", "with: space", "two\nlines"][selector % 5];
    match operation[0] % OPERATIONS {
        0 => {
            root.field("value")?.set(text)?;
            text.clone_into(&mut expected.value);
            text.clone_into(&mut expected.copy);
        }
        1 => {
            root.field("items")?.into_sequence()?.push(text)?;
            expected.items.push(text.to_owned());
        }
        2 => {
            let index = selector % (expected.items.len() + 1);
            root.field("items")?.into_sequence()?.insert(index, text)?;
            expected.items.insert(index, text.to_owned());
        }
        3 if !expected.items.is_empty() => {
            let index = selector % expected.items.len();
            root.field("items")?.into_sequence()?.remove(index)?;
            expected.items.remove(index);
        }
        4 => {
            root.field("map")?.into_mapping()?.set(key, text)?;
            expected.map.insert(key.to_owned(), text.to_owned());
        }
        5 => {
            root.field("map")?.into_mapping()?.remove(key)?;
            expected.map.remove(key);
        }
        6 => {
            let style = if selector % 2 == 0 {
                CollectionStyle::Block
            } else {
                CollectionStyle::Flow
            };
            // A flow parent cannot honor an earlier nested block-style request.
            if style == CollectionStyle::Flow {
                set_nested_style(root.reborrow(), style)?;
            }
            root.set_collection_style(style)?;
        }
        7 => root
            .field("value")?
            .set_scalar_style(ScalarStyle::DoubleQuoted)?,
        8 => {
            // These attachments are representable for every generated root.
            // Trailing comments after detached suffixes would be invalid.
            let placement = match selector % 4 {
                0 => CommentPlacement::Leading,
                1 => CommentPlacement::Inline,
                2 if !comments
                    .root
                    .iter()
                    .any(|(_, placement)| *placement == CommentPlacement::Detached) =>
                {
                    CommentPlacement::Trailing
                }
                _ => CommentPlacement::Detached,
            };
            if placement == CommentPlacement::Trailing && !comments.nested.is_empty() {
                set_nested_style(root.reborrow(), CollectionStyle::Flow)?;
            }
            if placement != CommentPlacement::Leading || comments.root.is_empty() {
                root.append_comment(" model", placement)?;
                comments.root.push((" model".to_owned(), placement));
            }
        }
        9 => {
            root.field("items")?.set([text])?;
            expected.items = vec![text.to_owned()];
        }
        10 => {
            root.field("map")?.set(BTreeMap::<String, String>::new())?;
            expected.map.clear();
        }
        11 => {
            root.field("map")?
                .into_mapping()?
                .set(key, "first")?
                .set(text)?;
            expected.map.insert(key.to_owned(), text.to_owned());
        }
        12 if !comments.root.is_empty() => {
            let index = selector % comments.root.len();
            root.remove_comment(index)?;
            comments.root.remove(index);
        }
        13 if !comments.root.is_empty() => {
            let index = selector % comments.root.len();
            root.replace_comment(index, " changed")?;
            " changed".clone_into(&mut comments.root[index].0);
        }
        14 => {
            nested(root)?.field("value")?.set("new\n\n")?;
            "new\n\n".clone_into(expected.nested.value_mut());
        }
        15 => {
            // Keep prior exact style requests compatible with this boundary.
            if comments
                .root
                .iter()
                .any(|(_, placement)| *placement == CommentPlacement::Trailing)
            {
                set_nested_style(root.reborrow(), CollectionStyle::Flow)?;
            }
            nested(root)?.append_comment(" nested", CommentPlacement::Detached)?;
            comments
                .nested
                .push((" nested".to_owned(), CommentPlacement::Detached));
        }
        16 if !comments.nested.is_empty() => {
            let index = selector % comments.nested.len();
            nested(root)?.remove_comment(index)?;
            comments.nested.remove(index);
        }
        17 => {
            let style = if selector % 2 == 0
                && (comments.nested.is_empty()
                    || !comments
                        .root
                        .iter()
                        .any(|(_, placement)| *placement == CommentPlacement::Trailing))
            {
                CollectionStyle::Block
            } else {
                CollectionStyle::Flow
            };
            if style == CollectionStyle::Block {
                root.set_collection_style(style)?;
            }
            set_nested_style(root, style)?;
        }
        18 => {
            nested(root)?.field("value")?.set(text)?;
            text.clone_into(expected.nested.value_mut());
        }
        19 => {
            root.append_comment(" temporary", CommentPlacement::Inline)?;
            root.remove_comment(comments.root.len())?;
        }
        20 => {
            let index = selector % (expected.items.len() + 1);
            let mut items = root.field("items")?.into_sequence()?;
            items.insert(index, text)?;
            items.remove(index)?;
        }
        21 => {
            // This key is outside the model's persistent key vocabulary.
            let mut map = root.field("map")?.into_mapping()?;
            map.set("__temporary__", text)?;
            map.remove("__temporary__")?;
        }
        22 => {
            root.field("map")?
                .into_mapping()?
                .entry(key)?
                .or_insert(text)?;
            expected
                .map
                .entry(key.to_owned())
                .or_insert_with(|| text.to_owned());
        }
        _ => {}
    }
    Ok(())
}

fn check(document: &Editable<Config>, expected: &Config, comments: &Comments) {
    assert!(!document.source().contains('\r'));
    assert_eq!(document.value(), expected);
    let syntax = document.syntax();
    let root = syntax.node(syntax.root()).unwrap();
    let actual: Vec<_> = root
        .comments()
        .iter()
        .map(|comment| (comment.text().to_owned(), comment.placement()))
        .collect();
    assert_eq!(actual, comments.root);
    let NodeKind::Mapping { entries, .. } = root.kind() else {
        unreachable!()
    };
    let field = |name: &str| {
        entries
            .iter()
            .find_map(|&(key, value)| {
                let NodeKind::Scalar { value: text, .. } = syntax.node(key).unwrap().kind() else {
                    return None;
                };
                (text == name).then_some(value)
            })
            .unwrap()
    };
    let mut nested = field("nested");
    while let NodeKind::Sequence { items, .. } = syntax.node(nested).unwrap().kind() {
        nested = items[0];
    }
    let actual: Vec<_> = syntax
        .node(nested)
        .unwrap()
        .comments()
        .iter()
        .map(|comment| (comment.text().to_owned(), comment.placement()))
        .collect();
    assert_eq!(actual, comments.nested);
    let value = field("value");
    assert_eq!(syntax.node(value).unwrap().anchor(), Some("a"));
    assert!(
        matches!(syntax.node(field("copy")).unwrap().kind(), NodeKind::Alias { target, .. } if *target == value)
    );
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 || data.len() > 1024 {
        return;
    }
    let controls = data[0];
    // Two header bytes select the layout and operation count, followed by
    // (operation, selector) pairs and an optional replacement string.
    let count = usize::from(data[1] % 64 + 1).min((data.len() - 2) / 2);
    let operations = &data[2..2 + count * 2];
    let payload = String::from_utf8_lossy(&data[2 + count * 2..]);
    let text =
        ["null", "first\n\nlast\n\n", "\r\n", "\u{feff}", &payload][usize::from(controls >> 4) % 5];
    // Compact mappings expose indentation inherited through enclosing sequences.
    let depth = usize::from(controls >> 6);
    let nested_source = format!("  {}value: 'old'", "- ".repeat(depth));
    let source = if controls & 1 == 0 {
        formatdoc! {r"
            value: &a old # stable
            copy: *a
            items: [x]
            map: {{key: value}}
            nested:
            {nested_source}
        "}
    } else {
        formatdoc! {r"
            ---
            value: &a |-
              old
            copy: *a
            items: [x]
            map: {{key: value}}
            nested:
            {nested_source}
            ...
        "}
    };
    let newline = ["\n", "\r\n", "\r"][usize::from(controls >> 1) % 3];
    let input = source.replace('\n', newline);
    let mut document = Editable::<Config>::parse(&input).unwrap();
    assert_eq!(document.source(), source);
    let mut nested = Nested::Mapping {
        value: "old".to_owned(),
    };
    for _ in 0..depth {
        nested = Nested::Sequence(vec![nested]);
    }
    let mut expected = Config {
        value: "old".to_owned(),
        copy: "old".to_owned(),
        items: vec!["x".to_owned()],
        map: BTreeMap::from([("key".to_owned(), "value".to_owned())]),
        nested,
    };
    let mut comments = Comments::default();
    let batch = [1, 4, 16][usize::from(controls >> 3) % 3];
    for operations in operations.chunks(batch * 2) {
        let canceled = operations
            .as_chunks::<2>()
            .0
            .iter()
            .all(|operation| matches!(operation[0] % OPERATIONS, 19..=21));
        let unchanged_source = canceled.then(|| document.source().to_owned());
        document
            .edit(|mut root| {
                for operation in operations.as_chunks::<2>().0 {
                    apply(
                        root.reborrow(),
                        &mut expected,
                        &mut comments,
                        operation,
                        text,
                    )?;
                }
                Ok(())
            })
            .unwrap();
        check(&document, &expected, &comments);
        if let Some(source) = unchanged_source {
            assert_eq!(
                document.source(),
                source,
                "canceled edits must not rewrite source"
            );
        }
    }
});
