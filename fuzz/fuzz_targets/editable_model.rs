#![no_main]

//! Valid operations must succeed and match an independent model after each batch.

use std::collections::BTreeMap;

use astral_yaml::comments::CommentPlacement;
use astral_yaml_dev::indoc;
use astral_yaml_serde::{
    Editable, Error,
    edit::{CollectionStyle, Editor, NodeKind, ScalarStyle},
};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    value: String,
    copy: String,
    items: Vec<String>,
    map: BTreeMap<String, String>,
    nested: Nested,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Nested {
    value: String,
}

#[derive(Default)]
struct Comments {
    root: Vec<(String, CommentPlacement)>,
    nested: Vec<(String, CommentPlacement)>,
}

fn apply(
    editor: &mut Editor<'_, '_>,
    expected: &mut Config,
    comments: &mut Comments,
    operation: &[u8; 2],
    text: &str,
) -> Result<(), Error> {
    let selector = usize::from(operation[1]);
    let key = ["null", "true", "01", "with: space", "two\nlines"][selector % 5];
    match operation[0] % 19 {
        0 => {
            editor.root_mut().field("value")?.set(text)?;
            text.clone_into(&mut expected.value);
            text.clone_into(&mut expected.copy);
        }
        1 => {
            editor.root_mut().field("items")?.push(text)?;
            expected.items.push(text.to_owned());
        }
        2 => {
            let index = selector % (expected.items.len() + 1);
            editor.root_mut().field("items")?.insert(index, text)?;
            expected.items.insert(index, text.to_owned());
        }
        3 if !expected.items.is_empty() => {
            let index = selector % expected.items.len();
            editor.root_mut().field("items")?.remove_item(index)?;
            expected.items.remove(index);
        }
        4 => {
            editor.root_mut().field("map")?.set_entry(key, text)?;
            expected.map.insert(key.to_owned(), text.to_owned());
        }
        5 => {
            editor.root_mut().field("map")?.remove_field(key)?;
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
                editor
                    .root_mut()
                    .field("nested")?
                    .set_collection_style(style)?;
            }
            editor.root_mut().set_collection_style(style)?;
        }
        7 => editor
            .root_mut()
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
                editor
                    .root_mut()
                    .field("nested")?
                    .set_collection_style(CollectionStyle::Flow)?;
            }
            let index = if placement == CommentPlacement::Leading {
                0
            } else {
                comments.root.len()
            };
            let syntax = editor.syntax_mut();
            syntax.insert_comment(&syntax.root(), index, " model", placement)?;
            comments
                .root
                .insert(index, (" model".to_owned(), placement));
        }
        9 => {
            editor.root_mut().field("items")?.set([text])?;
            expected.items = vec![text.to_owned()];
        }
        10 => {
            editor
                .root_mut()
                .field("map")?
                .set(BTreeMap::<String, String>::new())?;
            expected.map.clear();
        }
        11 => {
            editor
                .root_mut()
                .field("map")?
                .set_entry(key, "first")?
                .set(text)?;
            expected.map.insert(key.to_owned(), text.to_owned());
        }
        12 if !comments.root.is_empty() => {
            let index = selector % comments.root.len();
            editor.root_mut().remove_comment(index)?;
            comments.root.remove(index);
        }
        13 if !comments.root.is_empty() => {
            let index = selector % comments.root.len();
            editor.root_mut().replace_comment(index, " changed")?;
            " changed".clone_into(&mut comments.root[index].0);
        }
        14 => {
            let syntax = editor.syntax_mut();
            let nested = syntax
                .mapping_entries(&syntax.root())?
                .last()
                .unwrap()
                .1
                .clone();
            let value = syntax.mapping_entries(&nested)?[0].1.clone();
            syntax.replace(
                &value,
                if selector % 2 == 0 {
                    "|+\n  new\n\n"
                } else {
                    ">+\n  new\n\n"
                },
            )?;
            "new\n\n".clone_into(&mut expected.nested.value);
        }
        15 => {
            // Keep prior exact style requests compatible with this boundary.
            if comments
                .root
                .iter()
                .any(|(_, placement)| *placement == CommentPlacement::Trailing)
            {
                editor
                    .root_mut()
                    .field("nested")?
                    .set_collection_style(CollectionStyle::Flow)?;
            }
            editor
                .root_mut()
                .field("nested")?
                .append_comment(" nested", CommentPlacement::Detached)?;
            comments
                .nested
                .push((" nested".to_owned(), CommentPlacement::Detached));
        }
        16 if !comments.nested.is_empty() => {
            let index = selector % comments.nested.len();
            editor.root_mut().field("nested")?.remove_comment(index)?;
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
                editor.root_mut().set_collection_style(style)?;
            }
            editor
                .root_mut()
                .field("nested")?
                .set_collection_style(style)?;
        }
        18 => {
            editor
                .root_mut()
                .field("nested")?
                .field("value")?
                .set(text)?;
            text.clone_into(&mut expected.nested.value);
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
    let actual: Vec<_> = syntax
        .node(field("nested"))
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
    let source = if controls & 1 == 0 {
        indoc! {r"
            value: &a old # stable
            copy: *a
            items: [x]
            map: {key: value}
            nested:
              value: old
        "}
    } else {
        indoc! {r"
            ---
            value: &a |-
              old
            copy: *a
            items: [x]
            map: {key: value}
            nested:
              value: old
            ...
        "}
    };
    let newline = ["\n", "\r\n", "\r"][usize::from(controls >> 1) % 3];
    let input = source.replace('\n', newline);
    let mut document = Editable::<Config>::parse(&input).unwrap();
    assert_eq!(document.source(), source);
    let mut expected = Config {
        value: "old".to_owned(),
        copy: "old".to_owned(),
        items: vec!["x".to_owned()],
        map: BTreeMap::from([("key".to_owned(), "value".to_owned())]),
        nested: Nested {
            value: "old".to_owned(),
        },
    };
    let mut comments = Comments::default();
    let batch = [1, 4, 16][usize::from(controls >> 3) % 3];
    for operations in operations.chunks(batch * 2) {
        document
            .edit(|editor| {
                for operation in operations.as_chunks::<2>().0 {
                    apply(editor, &mut expected, &mut comments, operation, text)?;
                }
                Ok(())
            })
            .unwrap();
        check(&document, &expected, &comments);
    }
});
