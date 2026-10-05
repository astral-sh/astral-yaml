use astral_yaml_dev::edit_diff;
use astral_yaml_serde::{Editable, Tagged, edit::CollectionStyle};
use serde::Deserialize;

#[test]
fn unrelated_edits_preserve_empty_strings_and_tagged_payloads() {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    #[derive(Deserialize)]
    struct Config {
        plain: String,
        tagged: Tagged<String>,
        copy: String,
    }

    let source = "plain: &empty\ntagged: !custom\ncopy: *empty\nother: keep\n";
    let mut diffs = Vec::new();
    for operation in 0..3 {
        let mut document = Editable::<Config>::parse(source).unwrap();
        document
            .edit(|editor| {
                match operation {
                    0 => {
                        editor.root_mut().set_entry("enabled", true)?;
                    }
                    1 => {
                        editor.root_mut().remove_field("other")?;
                    }
                    _ => editor
                        .root_mut()
                        .set_collection_style(CollectionStyle::Flow)?,
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(document.value().plain, "");
        assert_eq!(document.value().tagged.value, "");
        assert_eq!(document.value().tagged.tag, "!custom");
        assert_eq!(document.value().copy, "");
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"));
}

#[test]
fn structural_edits_preserve_separate_block_scalar_comments() {
    #[derive(Deserialize)]
    struct Config {
        text: String,
    }

    let source = "text: &text # anchor\n  | # header\n    content\n";
    let mut document = Editable::<Config>::parse(source).unwrap();
    document
        .edit(|editor| {
            editor.root_mut().set_entry("enabled", true)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(document.value().text, "content\n");
    let stream = astral_yaml::parse(document.source()).unwrap();
    assert_eq!(
        stream
            .comments()
            .iter()
            .map(|comment| comment.text)
            .collect::<Vec<_>>(),
        [" anchor", " header"]
    );
    insta::assert_snapshot!(edit_diff(source, document.source()));
}

#[test]
fn insertion_uses_explicit_syntax_for_long_escaped_keys() {
    // Implicit keys: https://yaml.org/spec/1.2.2/#832-block-mappings
    let mut document = Editable::<serde_json::Value>::parse("existing: 1\n").unwrap();
    let key = "\u{1}".repeat(103);
    document
        .edit(|editor| {
            editor.root_mut().set_entry(&key, true)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(document.value()[&key], true);
    assert_eq!(document.value()["existing"], 1);
    assert!(document.source().contains("\n? "));
}
