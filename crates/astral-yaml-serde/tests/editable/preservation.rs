use super::support::{edit, edit_syntax};
use astral_yaml_dev::{edit_diff, formatdoc, indoc};
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

    let source = indoc! {r"
        plain: &empty
        tagged: !custom
        copy: *empty
        other: keep
    "};
    let mut diffs = Vec::new();
    for operation in 0..3 {
        let document = edit::<Config>(source, |editor| {
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
        });
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

    let source = indoc! {r"
        text: &text # anchor
          | # header
            content
    "};
    let document = edit::<Config>(source, |editor| {
        editor.root_mut().set_entry("enabled", true)?;
        Ok(())
    });
    assert_eq!(
        document.value().text,
        indoc! {r"
            content
        "}
    );
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
    let source = indoc! {r"
        existing: 1
    "};
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
    let source = indoc! {r"
        --- [old]
        ...
    "};
    let document = edit::<Vec<String>>(source, |editor| {
        let mut root = editor.root_mut();
        root.set(["new", "values"])?;
        root.set_collection_style(CollectionStyle::Block)
    });
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

    let source = indoc! {r"
        value

        # old
    "};
    let mut document = edit::<String>(source, |editor| {
        editor
            .root_mut()
            .append_comment(" new", CommentPlacement::Detached)
    });
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
        (
            indoc! {r"
                first
                second"},
            ScalarStyle::Plain,
        ),
        (
            indoc! {r"
                first
                second"},
            ScalarStyle::SingleQuoted,
        ),
        ("\n", ScalarStyle::Literal),
        ("\n", ScalarStyle::Folded),
    ] {
        let source = indoc! {r"
            'old'

        "};
        let document = edit::<String>(source, |editor| {
            let mut root = editor.root_mut();
            root.set(value)?;
            root.set_scalar_style(style)
        });
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
    for source in [
        indoc! {r"
            old
        "},
        indoc! {r"
            --- old
        "},
        r"  old
",
        "\u{feff}old\n",
        indoc! {r"
            &a old
        "},
    ] {
        for value in [
            "---",
            "...",
            "--- text",
            "... text",
            indoc! {r"
                ---
                text"},
        ] {
            let document = edit::<String>(source, |editor| editor.root_mut().set(value));
            assert_eq!(document.value(), value);
            if source
                == indoc! {r"
                    old
                "}
                || value == "---"
            {
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
    let source = indoc! {r"
        key: # header
          &a # anchor
          # before content
          old
    "};
    let document =
        edit::<serde_json::Value>(source, |editor| editor.root_mut().field("key")?.set("new"));
    assert_eq!(document.value()["key"], "new");
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     key: # header
       &a # anchor
    -  # before content
    -  old
    +    # before content
    +    new
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
        let document = edit::<serde_json::Value>(source, |editor| editor.root_mut().set(&value));
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
    for value in [
        serde_json::json!("new"),
        serde_json::json!(["a", "b"]),
        serde_json::json!({"a": "b"}),
    ] {
        let source = indoc! {r"
            # config
            %YAML 1.2
            ---
            ... # end
        "};
        let document = edit::<serde_json::Value>(source, |editor| editor.root_mut().set(&value));
        assert_eq!(document.value(), &value);
        diffs.push(edit_diff(source, document.source()));
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
                formatdoc! {r"
                    - old

                    # detached

                    - |{chomp}
                      text

                "}
            } else {
                formatdoc! {r"
                    a: old

                    # detached

                    b: |{chomp}
                      text

                "}
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
    for source in [
        indoc! {r"
            &root [old] # inline
        "},
        indoc! {r"
            !!seq [old] # inline
        "},
    ] {
        let document = edit::<Vec<String>>(source, |editor| {
            editor
                .root_mut()
                .set_collection_style(CollectionStyle::Block)?;
            editor.root_mut().push("new")?;
            Ok(())
        });
        assert_eq!(document.value(), &["old", "new"]);
        diffs.push(edit_diff(source, document.source()));
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
    "#);
}

#[test]
fn commented_fragments_refresh_the_typed_view() {
    let source = indoc! {r"
        a: old

        # tail
    "};
    let document = edit_syntax::<std::collections::BTreeMap<String, String>>(source, |editor| {
        editor.replace(
            &editor.root(),
            indoc! {r"
                # leading
                a: one

                # middle

                b: two
            "},
        )
    });
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

#[test]
fn typed_insertions_keep_flow_delimiter_comments() {
    let source = indoc! {r"
        [ # open
          old
        ] # close
    "};
    let document = edit::<Vec<String>>(source, |editor| {
        editor.root_mut().push("new")?;
        Ok(())
    });
    assert_eq!(document.value(), &["old", "new"]);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     [ # open
    -  old
    -] # close
    +  old, "new"] # close
    "#);
}

#[test]
fn typed_collection_edits_retain_outside_comments() {
    #[derive(Deserialize)]
    struct Config {
        outer: std::collections::BTreeMap<String, String>,
    }

    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            outer:
            # detached

              key: old
            unknown: keep
        "},
        indoc! {r"
            outer:
            # leading
              {key: old}
            unknown: keep
        "},
    ] {
        let document = edit::<Config>(source, |editor| {
            let mut outer = editor.root_mut().field("outer")?;
            outer.set_collection_style(CollectionStyle::Flow)?;
            outer.reborrow().set_entry("added", "new")?;
            Ok(())
        });
        assert_eq!(
            document.value().outer,
            std::collections::BTreeMap::from([
                ("key".to_owned(), "old".to_owned()),
                ("added".to_owned(), "new".to_owned()),
            ])
        );
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     outer:
    -# detached
    +  # detached
     ⏎
    -  key: old
    +  {key: old, "added": "new"}
     unknown: keep

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    -outer:
     # leading
    -  {key: old}
    +outer: {key: old, "added": "new"}
     unknown: keep
    "#);
}

#[test]
fn typed_collection_edits_use_document_comment_boundaries() {
    let source = indoc! {r"
        ---
        [ # open
          old
        ] # close
        ...
    "};
    let document = edit::<Vec<String>>(source, |editor| {
        let mut root = editor.root_mut();
        root.set_collection_style(CollectionStyle::Block)?;
        root.push("new")?;
        Ok(())
    });
    assert_eq!(document.value(), &["old", "new"]);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,4 @@
    ----
    -[ # open
    -  old
    -] # close
    -...
    +--- # open
    +- old
    +- "new"
    +... # close
    "#);
}

#[test]
fn typed_root_omission_refreshes_the_value_and_preserves_comments() {
    let source = indoc! {r"
        ---
        # leading
        old # inline
        ...
    "};
    let document = edit_syntax::<Option<String>>(source, |editor| {
        editor.replace(
            &editor.root(),
            indoc! {r"
                ---
            "},
        )
    });
    assert_eq!(document.value(), &None);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    ----
     # leading
    -old # inline
    +--- # inline
     ...
    ");
}

#[test]
fn typed_scalar_style_edits_keep_comments_out_of_the_value() {
    let source = indoc! {r"
          old
        # tail
    "};
    let document = edit::<String>(source, |editor| {
        editor.root_mut().set_scalar_style(ScalarStyle::Literal)
    });
    assert_eq!(document.value(), "old");
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    +  |2-
       old
     # tail
    ");
}
