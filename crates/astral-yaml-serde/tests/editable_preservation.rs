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
    let source = "--- [old]\n...\n";
    let mut document = Editable::<Vec<String>>::parse(source).unwrap();
    document
        .edit(|editor| {
            let mut root = editor.root_mut();
            root.set(["new", "values"])?;
            root.set_collection_style(CollectionStyle::Block)
        })
        .unwrap();
    assert_eq!(document.value().as_slice(), ["new", "values"]);
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    ---- [old]
    +--- ⏎
    +- "new"
    +- "values"
     ...
    "#);

    let source = "value\n\n# old\n";
    let mut document = Editable::<String>::parse(source).unwrap();
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
    assert_eq!(document.value(), "value");
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     value
     ⏎
     # old
    +# changed
    ");
}

#[test]
fn value_setters_can_request_multiline_scalar_styles() {
    let mut diffs = Vec::new();
    for (value, style) in [
        ("first\nsecond", ScalarStyle::Plain),
        ("first\nsecond", ScalarStyle::SingleQuoted),
        ("\n", ScalarStyle::Literal),
        ("\n", ScalarStyle::Folded),
    ] {
        let source = "'old'\n\n";
        let mut document = Editable::<String>::parse(source).unwrap();
        document
            .edit(|editor| {
                let mut root = editor.root_mut();
                root.set(value)?;
                root.set_scalar_style(style)
            })
            .unwrap();
        assert_eq!(document.value(), value);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -'old'
    +first
    +
    +  second
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -'old'
    +'first
    +
    +  second'
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -'old'
    -
    +|2+
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -'old'
    -
    +>2+
    +  ⏎
    ");
}

#[test]
fn typed_root_setters_distinguish_strings_from_document_markers() {
    // Document markers: https://yaml.org/spec/1.2.2/#912-document-markers
    let mut diffs = Vec::new();
    for source in ["old\n", "--- old\n", "  old\n", "\u{feff}old\n", "&a old\n"] {
        for value in ["---", "...", "--- text", "... text", "---\ntext"] {
            let mut document = Editable::<String>::parse(source).unwrap();
            document
                .edit(|editor| editor.root_mut().set(value))
                .unwrap();
            assert_eq!(document.value(), value);
            if source == "old\n" || value == "---" {
                diffs.push(edit_diff(source, document.source()));
            }
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"---"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"..."

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"--- text"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"... text"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"---\ntext"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    ---- old
    +--- ---

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -  old
    +  ---

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -﻿old
    +﻿"---"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -&a old
    +&a ---
    "#);
}

#[test]
fn typed_setters_preserve_comments_interleaved_with_properties() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source =
            "key: # header\n  &a # anchor\n  # before content\n  old\n".replace('\n', newline);
        let mut document = Editable::<serde_json::Value>::parse(&source).unwrap();
        document
            .edit(|editor| editor.root_mut().field("key")?.set("new"))
            .unwrap();
        assert_eq!(document.value()["key"], "new");
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     key: # header
       &a # anchor
    -  # before content
    -  old
    +    # before content
    +    new

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     key: # header␍
       &a # anchor␍
    -  # before content␍
    -  old␍
    +    # before content␍
    +    new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     key: # header␍   &a # anchor␍-  # before content␍-  old␍+    # before content␍+    new␍
    ");
}

#[test]
fn typed_setters_terminate_an_empty_roots_header_comment() {
    let source = "--- # keep";
    let mut diffs = Vec::new();
    for value in [
        serde_json::json!("new"),
        serde_json::json!(["new"]),
        serde_json::json!({"key": "new"}),
    ] {
        let mut document = Editable::<serde_json::Value>::parse(source).unwrap();
        document
            .edit(|editor| editor.root_mut().set(&value))
            .unwrap();
        assert_eq!(document.value(), &value);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # keep
    \ No newline at end of file
    +--- # keep
    +new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # keep
    \ No newline at end of file
    +--- # keep
    +["new"]
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # keep
    \ No newline at end of file
    +--- # keep
    +{"key": "new"}
    \ No newline at end of file
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn typed_setters_fill_empty_documents_without_consuming_markers() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for value in [
            serde_json::json!("new"),
            serde_json::json!(["a", "b"]),
            serde_json::json!({"a": "b"}),
        ] {
            let source = "# config\n%YAML 1.2\n---\n... # end\n".replace('\n', newline);
            let mut document = Editable::<serde_json::Value>::parse(&source).unwrap();
            document
                .edit(|editor| editor.root_mut().set(&value))
                .unwrap();
            assert_eq!(document.value(), &value);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config
     %YAML 1.2
     ---
    +new
     ... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config
     %YAML 1.2
     ---
    +["a", "b"]
     ... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config
     %YAML 1.2
     ---
    +{"a": "b"}
     ... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍
     %YAML 1.2␍
     ---␍
    +new␍
     ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍
     %YAML 1.2␍
     ---␍
    +["a", "b"]␍
     ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍
     %YAML 1.2␍
     ---␍
    +{"a": "b"}␍
     ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍ %YAML 1.2␍ ---␍+new␍ ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍ %YAML 1.2␍ ---␍+["a", "b"]␍ ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config␍ %YAML 1.2␍ ---␍+{"a": "b"}␍ ... # end␍
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn unrelated_collection_edits_preserve_chomping_around_detached_comments() {
    // Chomping: https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let mut diffs = Vec::new();
    for chomp in ["-", "", "+"] {
        for sequence in [false, true] {
            let source = if sequence {
                format!("- old\n\n# detached\n\n- |{chomp}\n  text\n\n")
            } else {
                format!("a: old\n\n# detached\n\nb: |{chomp}\n  text\n\n")
            };
            let mut document = Editable::<serde_json::Value>::parse(&source).unwrap();
            let mut expected = document.value().clone();
            if sequence {
                expected
                    .as_array_mut()
                    .unwrap()
                    .insert(0, serde_json::json!("new"));
            } else {
                expected["added"] = serde_json::json!("new");
            }
            document
                .edit(|editor| {
                    if sequence {
                        editor.root_mut().insert(0, "new")?;
                    } else {
                        editor.root_mut().set_entry("added", "new")?;
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(document.value(), &expected);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -2,6 +2,6 @@
     ⏎
     # detached
     ⏎
    -b: |-
    +b: |2-
       text
    -
    +"added": "new"

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    +- "new"
     - old
     ⏎
     # detached
     ⏎
    -- |-
    +- |2-
       text
    -

    --- before.yaml
    +++ after.yaml
    @@ -2,6 +2,6 @@
     ⏎
     # detached
     ⏎
    -b: |
    +b: |2
       text
    -
    +"added": "new"

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    +- "new"
     - old
     ⏎
     # detached
     ⏎
    -- |
    +- |2
       text
    -

    --- before.yaml
    +++ after.yaml
    @@ -2,6 +2,7 @@
     ⏎
     # detached
     ⏎
    -b: |+
    +b: |2+
       text
    -
    +  ⏎
    +"added": "new"

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +- "new"
     - old
     ⏎
     # detached
     ⏎
    -- |+
    +- |2+
       text
    -
    +  ⏎
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn typed_collection_edits_preserve_root_property_comments() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in ["&root [old] # inline\n", "!!seq [old] # inline\n"] {
            let source = source.replace('\n', newline);
            let mut document = Editable::<Vec<String>>::parse(&source).unwrap();
            document
                .edit(|editor| {
                    editor
                        .root_mut()
                        .set_collection_style(CollectionStyle::Block)?;
                    editor.root_mut().push("new")?;
                    Ok(())
                })
                .unwrap();
            assert_eq!(document.value(), &["old", "new"]);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -&root [old] # inline
    +&root # inline
    +- old
    +- "new"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -!!seq [old] # inline
    +!<tag:yaml.org,2002:seq> # inline
    +- old
    +- "new"

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -&root [old] # inline␍
    +&root # inline␍
    +- old␍
    +- "new"␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -!!seq [old] # inline␍
    +!<tag:yaml.org,2002:seq> # inline␍
    +- old␍
    +- "new"␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -&root [old] # inline␍+&root # inline␍+- old␍+- "new"␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -!!seq [old] # inline␍+!<tag:yaml.org,2002:seq> # inline␍+- old␍+- "new"␍
    "#);
}

#[test]
fn commented_fragments_refresh_the_typed_view() {
    let source = "a: old\n\n# tail\n";
    let mut document =
        Editable::<std::collections::BTreeMap<String, String>>::parse(source).unwrap();
    document
        .edit_syntax(|editor| {
            editor.replace(&editor.root(), "# leading\na: one\n\n# middle\n\nb: two\n")
        })
        .unwrap();
    assert_eq!(
        document.value(),
        &std::collections::BTreeMap::from([
            ("a".to_owned(), "one".to_owned()),
            ("b".to_owned(), "two".to_owned()),
        ])
    );
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,8 @@
    -a: old
    +# leading
    +a: one
    +
    +# middle
    +
    +b: two
     ⏎
     # tail
    ");
}
