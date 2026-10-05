use astral_yaml_dev::edit_diff;
use astral_yaml_serde::{
    Editable, Tagged,
    edit::{CollectionStyle, CommentPlacement, ScalarStyle},
};
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
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
    -plain: &empty
    -tagged: !custom
    +plain: &empty ⏎
    +tagged: !<!custom> ⏎
     copy: *empty
     other: keep
    +"enabled": true

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    -plain: &empty
    -tagged: !custom
    +plain: &empty ⏎
    +tagged: !<!custom> ⏎
     copy: *empty
    -other: keep

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1 @@
    -plain: &empty
    -tagged: !custom
    -copy: *empty
    -other: keep
    +{plain: &empty , tagged: !<!custom> , copy: *empty, other: keep}
    "#);
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
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     text: &text # anchor
    -  | # header
    -    content
    +  |2 # header
    +  content
    +"enabled": true
    "#);
}

#[test]
fn insertion_uses_explicit_syntax_for_long_escaped_keys() {
    // Implicit keys: https://yaml.org/spec/1.2.2/#832-block-mappings
    let source = "existing: 1\n";
    let mut document = Editable::<serde_json::Value>::parse(source).unwrap();
    let key = "\u{1}".repeat(103);
    document
        .edit(|editor| {
            editor.root_mut().set_entry(&key, true)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(document.value()[&key], true);
    assert_eq!(document.value()["existing"], 1);
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
     existing: 1
    +? "\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001"
    +: true
    "#);
}

#[test]
fn typed_root_edits_preserve_document_framing_and_comment_order() {
    let mut document = Editable::<Vec<String>>::parse("--- [old]\n...\n").unwrap();
    document
        .edit(|editor| {
            let mut root = editor.root_mut();
            root.set(["new", "values"])?;
            root.set_collection_style(CollectionStyle::Block)
        })
        .unwrap();
    insta::assert_debug_snapshot!((document.value(), document.source()), @r#"
    (
        [
            "new",
            "values",
        ],
        "--- \n- \"new\"\n- \"values\"\n...\n",
    )
    "#);

    let mut document = Editable::<String>::parse("value\n\n# old\n").unwrap();
    document
        .edit(|editor| {
            editor
                .root_mut()
                .append_comment(" new", CommentPlacement::Detached)
        })
        .unwrap();
    document
        .edit(|editor| editor.root_mut().replace_comment(1, " changed"))
        .unwrap();
    insta::assert_debug_snapshot!((document.value(), document.source()), @r#"
    (
        "value",
        "value\n\n# old\n# changed\n",
    )
    "#);
}

#[test]
fn value_setters_can_request_multiline_scalar_styles() {
    let mut outputs = Vec::new();
    for (value, style) in [
        ("first\nsecond", ScalarStyle::Plain),
        ("first\nsecond", ScalarStyle::SingleQuoted),
        ("\n", ScalarStyle::Literal),
        ("\n", ScalarStyle::Folded),
    ] {
        let mut document = Editable::<String>::parse("'old'\n\n").unwrap();
        document
            .edit(|editor| {
                let mut root = editor.root_mut();
                root.set(value)?;
                root.set_scalar_style(style)
            })
            .unwrap();
        outputs.push((document.value().clone(), document.into_yaml()));
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        (
            "first\nsecond",
            "first\n\n  second\n\n",
        ),
        (
            "first\nsecond",
            "'first\n\n  second'\n\n",
        ),
        (
            "\n",
            "|2+\n  \n",
        ),
        (
            "\n",
            ">2+\n  \n",
        ),
    ]
    "#);
}
