use test_case::{test_case, test_matrix};

use super::support::edit;
use astral_yaml_dev::{edit_diff, indoc};
use astral_yaml_serde::edit::{CollectionStyle, CommentPlacement, ScalarStyle};
use astral_yaml_serde::{Editable, Tagged};
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
        let document = edit::<Config>(source, |mut root| {
            match operation {
                0 => {
                    root.into_mapping()?.set("enabled", true)?;
                }
                1 => {
                    root.into_mapping()?.remove("other")?;
                }
                _ => root.set_collection_style(CollectionStyle::Flow)?,
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
    @@ -1,4 +1,5 @@
    -plain: &empty
    -tagged: !custom
    +plain: &empty ⏎
    +tagged: !<!custom> ⏎
     copy: *empty
     other: keep
    +"enabled": true

    @@ -1,4 +1,3 @@
    -plain: &empty
    -tagged: !custom
    +plain: &empty ⏎
    +tagged: !<!custom> ⏎
     copy: *empty
    -other: keep

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
    // The comment after the anchor and the one on the scalar header both precede the content.
    // https://yaml.org/spec/1.2.2/#67-separation-lines
    // https://yaml.org/spec/1.2.2/#811-block-scalar-headers
    #[derive(Deserialize)]
    struct Config {
        text: String,
    }

    let source = indoc! {r"
        text: &text # anchor
          | # header
            content
    "};
    let document = edit::<Config>(source, |root| {
        root.into_mapping()?.set("enabled", true)?;
        Ok(())
    });
    assert_eq!(
        document.value().text,
        indoc! {r"
            content
        "}
    );
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    @@ -1,3 +1,4 @@
     text: &text # anchor
       | # header
    -    content
    +  content
    +"enabled": true
    "#);
}

#[test]
fn insertion_uses_explicit_syntax_for_long_escaped_keys() {
    // Implicit keys: https://yaml.org/spec/1.2.2/#822-block-mappings
    let key = "\u{1}".repeat(171);
    let document = edit::<serde_json::Value>("existing: 1\n", |root| {
        root.into_mapping()?.set(&key, true)?;
        Ok(())
    });
    assert_eq!(document.value()[&key], true);
    assert_eq!(document.value()["existing"], 1);

    let escaped = r"\u0001".repeat(171);
    assert_eq!(
        document.source(),
        format!("existing: 1\n? \"{escaped}\"\n: true\n")
    );
}

#[test]
fn typed_root_edits_preserve_document_framing_and_comment_order() {
    // Start and end markers delimit the editable document.
    // https://yaml.org/spec/1.2.2/#912-document-markers
    let source = indoc! {r"
        --- [old]
        ...
    "};
    let document = edit::<Vec<String>>(source, |mut root| {
        root.set(["new", "values"])?;
        root.set_collection_style(CollectionStyle::Block)
    });
    assert_eq!(document.value().as_slice(), ["new", "values"]);
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
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
    let mut document = edit::<String>(source, |mut root| {
        root.append_comment(" new", CommentPlacement::Detached)
    });
    document
        .edit(|mut root| root.replace_comment(1, " changed"))
        .unwrap();
    assert_eq!(document.value(), "value");
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,3 +1,4 @@
     value
     ⏎
     # old
    +# changed
    ");
}

#[test_case("first\nsecond", ScalarStyle::Plain, "first\n\n  second\n\n"; "plain")]
#[test_case("first\nsecond", ScalarStyle::SingleQuoted, "'first\n\n  second'\n\n"; "single quoted")]
#[test_case("\n", ScalarStyle::Literal, "|3+\n  \n"; "literal")]
#[test_case("\n", ScalarStyle::Folded, ">3+\n  \n"; "folded")]
fn value_setters_can_request_multiline_scalar_styles(
    value: &str,
    style: ScalarStyle,
    expected: &str,
) {
    // Plain and quoted styles fold line breaks. The block cases keep the trailing break.
    // https://yaml.org/spec/1.2.2/#65-line-folding
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let document = edit::<String>("'old'\n\n", |mut root| {
        root.set(value)?;
        root.set_scalar_style(style)
    });
    assert_eq!(document.value(), value);
    assert_eq!(document.source(), expected);
}

// Document markers: https://yaml.org/spec/1.2.2/#912-document-markers
#[test_matrix(
    ["", "--- ", "  ", "\u{feff}", "&a "],
    ["---", "...", "--- text", "... text", "---\ntext"]
)]
fn typed_root_setters_distinguish_strings_from_document_markers(prefix: &str, value: &str) {
    let document = edit::<String>(&format!("{prefix}old\n"), |mut root| root.set(value));

    let expected = match prefix {
        "" | "\u{feff}" => format!("{value:?}"),
        "  " => value.replace('\n', "\n\n    "),
        _ => value.replace('\n', "\n\n  "),
    };
    assert_eq!(document.source(), format!("{prefix}{expected}\n"));
    assert_eq!(document.value(), value);
}

#[test]
fn typed_setters_preserve_comments_interleaved_with_properties() {
    // Separation lines can appear between node properties and content.
    // https://yaml.org/spec/1.2.2/#67-separation-lines
    // https://yaml.org/spec/1.2.2/#69-node-properties
    let source = indoc! {r"
        key: # header
          &a # anchor
          # before content
          old
    "};
    let document = edit::<serde_json::Value>(source, |root| root.field("key")?.set("new"));
    assert_eq!(document.value()["key"], "new");
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    @@ -1,4 +1,4 @@
     key: # header
       &a # anchor
    -  # before content
    -  old
    +    # before content
    +    new
    ");
}

#[test_case(&serde_json::json!("new"), "new"; "scalar")]
#[test_case(&serde_json::json!(["new"]), r#"["new"]"#; "sequence")]
#[test_case(&serde_json::json!({"key": "new"}), r#"{"key": "new"}"#; "mapping")]
fn typed_setters_terminate_an_empty_roots_header_comment(value: &serde_json::Value, yaml: &str) {
    // The comment on the marker line ends before the new content begins.
    // https://yaml.org/spec/1.2.2/#66-comments
    // https://yaml.org/spec/1.2.2/#914-explicit-documents
    let source = "--- # keep";
    let document = edit::<serde_json::Value>(source, |mut root| root.set(value));
    assert_eq!(document.source(), format!("{source}\n{yaml}"));
    assert_eq!(document.value(), value);
}

#[test_case(&serde_json::json!("new"), "new"; "scalar")]
#[test_case(&serde_json::json!(["a", "b"]), r#"["a", "b"]"#; "sequence")]
#[test_case(&serde_json::json!({"a": "b"}), r#"{"a": "b"}"#; "mapping")]
fn typed_setters_fill_empty_documents_without_consuming_markers(
    value: &serde_json::Value,
    yaml: &str,
) {
    // An explicit document can be empty. The end marker is outside its content.
    // https://yaml.org/spec/1.2.2/#914-explicit-documents
    // https://yaml.org/spec/1.2.2/#912-document-markers
    let prefix = "# config\n%YAML 1.2\n---\n";
    let suffix = "... # end\n";
    let source = format!("{prefix}{suffix}");
    let document = edit::<serde_json::Value>(&source, |mut root| root.set(value));
    assert_eq!(document.source(), format!("{prefix}{yaml}\n{suffix}"));
    assert_eq!(document.value(), value);
}

#[test_case(false, "-", ""; "mapping strip")]
#[test_case(false, "", ""; "mapping clip")]
#[test_case(false, "+", "  \n"; "mapping keep")]
#[test_case(true, "-", ""; "sequence strip")]
#[test_case(true, "", ""; "sequence clip")]
#[test_case(true, "+", "  \n"; "sequence keep")]
fn unrelated_collection_edits_preserve_chomping_around_detached_comments(
    sequence: bool,
    chomp: &str,
    trailing: &str,
) {
    // Chomping: https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let (first, indicator) = if sequence {
        ("- old", "-")
    } else {
        ("a: old", "b:")
    };
    let prefix = format!("{first}\n\n# detached\n\n{indicator} ");
    let source = format!("{prefix}|{chomp}\n  text\n\n");
    let mut document = Editable::<serde_json::Value>::parse(&source).unwrap();
    let mut expected = document.value().clone();

    let block = format!("{prefix}|{chomp}\n  text\n{trailing}");
    let yaml = if sequence {
        document
            .edit(|root| root.into_sequence()?.insert(0, "new").map(|_| ()))
            .unwrap();
        expected
            .as_array_mut()
            .unwrap()
            .insert(0, serde_json::json!("new"));
        format!("- \"new\"\n{block}")
    } else {
        document
            .edit(|root| root.into_mapping()?.set("added", "new").map(|_| ()))
            .unwrap();
        expected["added"] = serde_json::json!("new");
        format!("{block}\"added\": \"new\"\n")
    };
    assert_eq!(document.source(), yaml);
    assert_eq!(document.value(), &expected);
}

#[test_case("&root", "&root"; "anchor")]
#[test_case("!!seq", "!<tag:yaml.org,2002:seq>"; "tag")]
fn typed_collection_edits_preserve_root_property_comments(property: &str, emitted: &str) {
    // A tag or anchor can precede a collection on a separate line.
    // https://yaml.org/spec/1.2.2/#69-node-properties
    let source = format!("{property} [old] # inline\n");
    let document = edit::<Vec<String>>(&source, |mut root| {
        root.set_collection_style(CollectionStyle::Block)?;
        root.into_sequence()?.push("new")?;
        Ok(())
    });
    assert_eq!(
        document.source(),
        format!("{emitted} # inline\n- old\n- \"new\"\n")
    );
    assert_eq!(document.value(), &["old", "new"]);
}

#[test]
fn typed_insertions_keep_flow_delimiter_comments() {
    // Flow collections allow separation lines around entries and delimiters.
    // https://yaml.org/spec/1.2.2/#74-flow-collection-styles
    // https://yaml.org/spec/1.2.2/#67-separation-lines
    let source = indoc! {r"
        [ # open
          old
        ] # close
    "};
    let document = edit::<Vec<String>>(source, |root| {
        root.into_sequence()?.push("new")?;
        Ok(())
    });
    assert_eq!(document.value(), &["old", "new"]);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @r#"
    @@ -1,3 +1,2 @@
     [ # open
    -  old
    -] # close
    +  old, "new"] # close
    "#);
}

#[test_case(
    indoc! {r"
        outer:
        # detached

          key: old
        unknown: keep
    "}
    => with |diff| insta::assert_snapshot!(diff, @r#"
        @@ -1,5 +1,5 @@
         outer:
        -# detached
        +  # detached
         ⏎
        -  key: old
        +  {key: old, "added": "new"}
         unknown: keep
    "#);
    "detached comment"
)]
#[test_case(
    indoc! {r"
        outer:
        # leading
          {key: old}
        unknown: keep
    "}
    => with |diff| insta::assert_snapshot!(diff, @r#"
        @@ -1,4 +1,3 @@
        -outer:
         # leading
        -  {key: old}
        +outer: {key: old, "added": "new"}
         unknown: keep
    "#);
    "leading comment"
)]
fn typed_collection_edits_retain_outside_comments(source: &str) -> String {
    #[derive(Deserialize)]
    struct Config {
        outer: std::collections::BTreeMap<String, String>,
    }

    let document = edit::<Config>(source, |root| {
        let mut outer = root.field("outer")?.into_mapping()?;
        outer.set_collection_style(CollectionStyle::Flow)?;
        outer.set("added", "new")?;
        Ok(())
    });
    assert_eq!(
        document.value().outer,
        std::collections::BTreeMap::from([
            ("key".to_owned(), "old".to_owned()),
            ("added".to_owned(), "new".to_owned()),
        ])
    );
    edit_diff(source, document.source())
}

#[test]
fn typed_collection_edits_use_document_comment_boundaries() {
    // A document suffix may carry the closing comment.
    // https://yaml.org/spec/1.2.2/#rule-l-document-suffix
    let source = indoc! {r"
        ---
        [ # open
          old
        ] # close
        ...
    "};
    let document = edit::<Vec<String>>(source, |root| {
        let mut root = root.into_sequence()?;
        root.set_collection_style(CollectionStyle::Block)?;
        root.push("new")?;
        Ok(())
    });
    assert_eq!(document.value(), &["old", "new"]);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @r#"
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
fn setting_root_to_null_refreshes_the_value_and_preserves_comments() {
    // The plain scalar `null` resolves to the core null tag.
    // https://yaml.org/spec/1.2.2/#1032-tag-resolution
    let source = indoc! {r"
        ---
        # leading
        old # inline
        ...
    "};
    let document = edit::<Option<String>>(source, |mut root| root.set(None::<String>));
    assert_eq!(document.value(), &None);
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    @@ -1,4 +1,4 @@
     ---
     # leading
    -old # inline
    +null # inline
     ...
    ");
}

#[test]
fn typed_scalar_style_edits_keep_comments_out_of_the_value() {
    // Indentation separates the trailing comment from the literal scalar.
    // https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let source = indoc! {r"
          old
        # tail
    "};
    let document = edit::<String>(source, |mut root| {
        root.set_scalar_style(ScalarStyle::Literal)
    });
    assert_eq!(document.value(), "old");
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,3 @@
    +  |-
       old
     # tail
    ");
}

// https://yaml.org/spec/1.2.2/#51-character-set
#[test_case("'old'", '\u{7f}'; "DEL in a single-quoted edit")]
#[test_case("'old'", '\u{80}'; "C1 control in a single-quoted edit")]
#[test_case("\"old\"", '\u{fffe}'; "double-quoted noncharacter")]
#[test_case("old", '\u{ffff}'; "plain fallback")]
#[test_case("'old'", '\u{feff}'; "content byte order mark")]
fn edited_scalars_escape_non_printable_characters(source: &str, character: char) {
    let value = format!("a{character}b");
    let mut document = Editable::<String>::parse(source).unwrap();
    document.edit(|mut root| root.set(&value)).unwrap();
    let yaml = document.into_yaml();
    assert!(!yaml.contains(character));
    assert_eq!(astral_yaml_serde::from_str::<String>(&yaml).unwrap(), value);
}

#[test]
fn normalization_preserves_an_escaped_noncharacter() {
    // A hexadecimal escape can represent a character excluded from printable YAML.
    // https://yaml.org/spec/1.2.2/#51-character-set
    // https://yaml.org/spec/1.2.2/#57-escaped-characters
    let document = Editable::<String>::parse(r#""a\uFFFFb""#).unwrap();
    let yaml = document.syntax().normalized_yaml().unwrap();
    assert!(!yaml.contains('\u{ffff}'));
    assert_eq!(
        astral_yaml_serde::from_str::<String>(&yaml).unwrap(),
        "a\u{ffff}b"
    );
}

#[test_case("'a\u{80}b'"; "single quoted")]
#[test_case("\"a\u{80}b\""; "double quoted")]
fn loading_reader_only_characters_preserves_unrelated_source(scalar: &str) {
    // The JSON compatibility rule permits additional literal characters in quoted input.
    // https://yaml.org/spec/1.2.2/#51-character-set
    let source = format!("keep : [x,   y]\nitem: &s {scalar} # data\ncopy: *s\n");
    let document = astral_yaml_serde::EditableYaml::parse(&source).unwrap();
    assert_eq!(
        document.into_yaml(),
        source.replace(scalar, r#""a\u0080b""#)
    );
}

#[test]
fn escaping_a_reader_only_character_can_require_an_explicit_key() {
    // The implicit key limit counts characters in the YAML source, including escape sequences.
    // https://yaml.org/spec/1.2.2/#822-block-mappings
    let key = "\u{7f}".repeat(171);
    let source = format!("'{key}': value\n");
    let document = Editable::<std::collections::BTreeMap<String, String>>::parse(&source).unwrap();
    assert_eq!(document.value()[&key], "value");
    assert_eq!(
        document.into_yaml(),
        format!("? \"{}\"\n: value\n", r"\u007F".repeat(171))
    );
}

#[test]
fn an_explicit_single_quoted_style_cannot_publish_a_non_printable_character() {
    // Backslash escapes for non-printable characters require double quotes.
    // https://yaml.org/spec/1.2.2/#57-escaped-characters
    // https://yaml.org/spec/1.2.2/#732-single-quoted-style
    let mut document = Editable::<String>::parse("'old'").unwrap();
    let error = document
        .edit(|mut root| {
            root.set("a\u{80}b")?;
            root.set_scalar_style(ScalarStyle::SingleQuoted)
        })
        .unwrap_err();
    assert!(error.message.contains("requested style"));
    assert_eq!(document.source(), "'old'");
}
