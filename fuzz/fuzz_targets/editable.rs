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
    assert_eq!(document.to_yaml(), source);

    for &operation in operations {
        let before = document.to_yaml();
        let value_before = to_string(document.value());
        let result = document.edit(|editor| {
            let node = select(editor, operation)?;
            match operation % 8 {
                0 => editor.replace(&node, if operation & 8 == 0 { "42" } else { "'null'" }),
                1 => editor.insert_item(&node, 0, "[1, 'two']"),
                2 => editor.remove_item(&node, 0),
                3 => editor.insert_entry(&node, 0, "added", "true"),
                4 => editor.remove_entry(&node, 0),
                5 => editor.set_scalar_style(&node, ScalarStyle::SingleQuoted),
                6 => editor.set_collection_style(&node, CollectionStyle::Block),
                _ => editor.insert_comment(&node, 0, " fuzz", CommentPlacement::Leading),
            }
        });
        if result.is_err() {
            assert_eq!(document.to_yaml(), before);
            assert_eq!(to_string(document.value()), value_before);
        } else {
            let source = document.to_yaml();
            let decoded: Value = from_str(&source).unwrap();
            assert_eq!(to_string(document.value()), to_string(&decoded));
        }
    }
});
