#![no_main]

use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, ScalarStyle, Source};
use astral_yaml_dev::Value;
use astral_yaml_serde::edit::{Cursor, NodeKind};
use astral_yaml_serde::{Editable, Error, from_str, to_string};
use libfuzzer_sys::fuzz_target;

// YAML 1.2.2 §5.1, production [1]. The reader also accepts some characters
// in quoted scalars that a writer must escape.
fn yaml_printable(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | ' '..='~' | '\u{85}' | '\u{a0}'..='\u{fffd}' | '\u{10000}'..)
}

fn check_output(source: &str) {
    assert!(!source.contains('\r'));
    assert!(source.chars().all(yaml_printable));
    Source::from_bytes(source.as_bytes()).unwrap();
}

fn edit_selected(cursor: Cursor<'_, '_>, selector: u8, depth: usize) -> Result<(), Error> {
    if depth < usize::from(selector >> 6) {
        let index = usize::from(selector >> (depth * 2));
        match cursor.node()?.kind() {
            NodeKind::Sequence { items, .. } if !items.is_empty() => {
                let index = index % items.len();
                return edit_selected(cursor.item(index)?, selector, depth + 1);
            }
            NodeKind::Mapping { entries, .. } if !entries.is_empty() => {
                let index = index % entries.len();
                let mut mapping = cursor.into_mapping()?;
                let child = if selector & 16 == 0 {
                    mapping.value_at(index)?
                } else {
                    mapping.key_at(index)?
                };
                return edit_selected(child, selector, depth + 1);
            }
            _ => {}
        }
    }
    let mut node = cursor;
    match selector % 8 {
        0 if selector & 8 == 0 => node.set(42),
        0 => node.set("null"),
        1 => node.into_sequence()?.insert(0, (1, "two")).map(|_| ()),
        2 => node.into_sequence()?.remove(0),
        3 => node.into_mapping()?.set("added", true).map(|_| ()),
        4 => node.into_mapping()?.remove_at(0),
        5 => node.set_scalar_style(ScalarStyle::SingleQuoted),
        6 => node.set_collection_style(CollectionStyle::Block),
        _ => {
            let placement = match (selector >> 3) & 3 {
                0 => CommentPlacement::Leading,
                1 => CommentPlacement::Inline,
                2 => CommentPlacement::Trailing,
                _ => CommentPlacement::Detached,
            };
            let index = usize::from(selector >> 6) % (node.node()?.comments().len() + 1);
            node.insert_comment(index, " fuzz", placement)
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 || data.len() > 65_536 {
        return;
    }
    let (operations, input) = data.split_at(8);
    let Ok(source) = std::str::from_utf8(input) else {
        return;
    };
    let Ok(mut document) = Editable::<Value>::parse(source) else {
        return;
    };
    let mut expected = source.replace("\r\n", "\n").replace('\r', "\n");
    if expected.starts_with(|c: char| !c.is_ascii() && c != '\u{feff}') {
        expected.insert(0, '\u{feff}');
    }
    check_output(document.source());
    if expected.chars().all(yaml_printable) {
        assert_eq!(document.source(), expected);
    }
    let stream_marks = expected.contains('\u{feff}').then(|| {
        astral_yaml::parse(&expected)
            .unwrap()
            .byte_order_marks()
            .len()
    });
    if let Some(count) = stream_marks {
        assert_eq!(
            astral_yaml::parse(document.source())
                .unwrap()
                .byte_order_marks()
                .len(),
            count
        );
    }
    let original: Value = from_str(source).unwrap();
    assert_eq!(to_string(document.value()), to_string(&original));

    for &operation in operations {
        let before = document.to_yaml();
        let value_before = to_string(document.value());
        let result = if operation & 32 == 0 {
            document.edit(|root| edit_selected(root, operation, 0))
        } else {
            document.edit(|mut root| {
                let index = usize::from(operation >> 6);
                match operation % 8 {
                    0 if operation & 8 == 0 => root.set("false")?,
                    0 => root.set(false)?,
                    1 => {
                        root.into_sequence()?.push([1, 2])?.item(0)?.set("null")?;
                    }
                    2 => root.into_sequence()?.remove(index)?,
                    3 => {
                        root.into_mapping()?
                            .set("added", [operation])?
                            .into_sequence()?
                            .push(operation)?;
                    }
                    4 => {
                        if let Some(mut added) = root.get("added")? {
                            added.set(true)?;
                            added.append_comment(" fuzz", CommentPlacement::Inline)?;
                        }
                    }
                    5 => root.item(index)?.set("replacement")?,
                    6 => {
                        root.into_mapping()?.remove("added")?;
                    }
                    _ => {
                        root.set([1, 2])?;
                        let mut items = root.into_sequence()?;
                        items.insert(index, "inserted")?;
                        let mut root = items.into_cursor();
                        root.set(["first", "second"])?;
                        let mut items = root.into_sequence()?;
                        items.item(1)?.set("last")?;
                        items.set_collection_style(CollectionStyle::Flow)?;
                    }
                }
                Ok(())
            })
        };
        if result.is_err() {
            assert_eq!(document.to_yaml(), before);
            assert_eq!(to_string(document.value()), value_before);
        } else {
            let source = document.to_yaml();
            check_output(&source);
            if let Some(count) = stream_marks {
                assert_eq!(
                    astral_yaml::parse(&source)
                        .unwrap()
                        .byte_order_marks()
                        .len(),
                    count
                );
            }
            let decoded: Value = from_str(&source).unwrap();
            assert_eq!(to_string(document.value()), to_string(&decoded));
        }
    }
});
