#![no_main]

use astral_yaml::{
    CollectionStyle, ScalarStyle,
    comments::CommentPlacement,
    edit::{Editor, Handle, NodeKind},
};
use astral_yaml_dev::Value;
use astral_yaml_serde::{Editable, from_str, to_string};
use libfuzzer_sys::fuzz_target;

fn select(editor: &Editor<'_>, selector: u8) -> Result<Handle, astral_yaml::Error> {
    let mut handle = editor.root();
    for depth in 0..usize::from(selector >> 6) {
        let index = usize::from(selector >> (depth * 2));
        handle = match editor.node(&handle)?.kind() {
            NodeKind::Sequence { items, .. } if !items.is_empty() => {
                let items = editor.sequence_items(&handle)?;
                items[index % items.len()].clone()
            }
            NodeKind::Mapping { entries, .. } if !entries.is_empty() => {
                let entries = editor.mapping_entries(&handle)?;
                let (key, value) = &entries[index % entries.len()];
                if selector & 16 == 0 {
                    value.clone()
                } else {
                    key.clone()
                }
            }
            _ => break,
        };
    }
    Ok(handle)
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
    assert_eq!(
        document.source(),
        source.replace("\r\n", "\n").replace('\r', "\n")
    );
    let original: Value = from_str(source).unwrap();
    assert_eq!(to_string(document.value()), to_string(&original));

    for &operation in operations {
        let before = document.to_yaml();
        let value_before = to_string(document.value());
        let result = if operation & 32 == 0 {
            document.edit_syntax(|editor| {
                let node = select(editor, operation)?;
                match operation % 8 {
                    0 => editor.replace(&node, if operation & 8 == 0 { "42" } else { "'null'" }),
                    1 => editor.insert_item(&node, 0, "[1, 'two']").map(|_| ()),
                    2 => editor.remove_item(&node, 0),
                    3 => editor.insert_entry(&node, 0, "added", "true").map(|_| ()),
                    4 => editor.remove_entry(&node, 0),
                    5 => editor.set_scalar_style(&node, ScalarStyle::SingleQuoted),
                    6 => editor.set_collection_style(&node, CollectionStyle::Block),
                    _ => {
                        let placement = match (operation >> 3) & 3 {
                            0 => CommentPlacement::Leading,
                            1 => CommentPlacement::Inline,
                            2 => CommentPlacement::Trailing,
                            _ => CommentPlacement::Detached,
                        };
                        let index = if placement == CommentPlacement::Leading {
                            0
                        } else {
                            editor.node(&node)?.comments().len()
                        };
                        editor.insert_comment(&node, index, " fuzz", placement)
                    }
                }
            })
        } else {
            document.edit(|editor| {
                let index = usize::from(operation >> 6);
                match operation % 8 {
                    0 if operation & 8 == 0 => editor.root_mut().set("false")?,
                    0 => editor.root_mut().set(false)?,
                    1 => {
                        editor
                            .root_mut()
                            .into_sequence()?
                            .push([1, 2])?
                            .item(0)?
                            .set("null")?;
                    }
                    2 => editor.root_mut().into_sequence()?.remove(index)?,
                    3 => {
                        editor
                            .root_mut()
                            .into_mapping()?
                            .set("added", [operation])?
                            .into_sequence()?
                            .push(operation)?;
                    }
                    4 => {
                        if let Some(mut added) = editor.root_mut().get("added")? {
                            added.set(true)?;
                            added.append_comment(" fuzz", CommentPlacement::Inline)?;
                        }
                    }
                    5 => editor.root_mut().item(index)?.set("replacement")?,
                    6 => {
                        editor.root_mut().into_mapping()?.remove("added")?;
                    }
                    _ => {
                        let mut root = editor.root_mut();
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
            assert!(!source.contains('\r'));
            let decoded: Value = from_str(&source).unwrap();
            assert_eq!(to_string(document.value()), to_string(&decoded));
        }
    }
});
