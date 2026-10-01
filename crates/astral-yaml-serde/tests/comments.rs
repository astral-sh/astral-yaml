//! Comment syntax: <https://yaml.org/spec/1.2.2/#66-comments>.
//! Ownership and aggregation are astral-yaml-serde API behavior.

use astral_yaml_serde::{
    AttachedComment, CommentPlacement, Spanned, from_document, from_str, to_string,
};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write,
};

use CommentPlacement::{Detached, Inline, Leading, Trailing};

type Fields<'a> = BTreeMap<String, Spanned<'a, String>>;
type ComplexKey<'a> = Spanned<'a, Vec<Spanned<'a, String>>>;

fn comments<'a, T>(value: &Spanned<'a, T>) -> Vec<(&'a str, CommentPlacement)> {
    value
        .comments
        .iter()
        .map(|attached| (attached.comment.text, attached.placement))
        .collect()
}

fn comment_snapshot(source: &str, groups: &[(&str, &[AttachedComment<'_>])]) -> String {
    let stream = astral_yaml::parse(source).unwrap();
    let mut output = String::new();

    for (name, comments) in groups {
        writeln!(output, "{name}:").unwrap();

        if comments.is_empty() {
            writeln!(output, "  (none)").unwrap();
        }

        for attached in *comments {
            let comment = &attached.comment;
            let (start_line, start_column) = stream.location(comment.span.start()).unwrap();
            let (end_line, end_column) = stream.location(comment.span.end()).unwrap();
            writeln!(
                output,
                "  {:?} @{start_line}:{start_column}..{end_line}:{end_column} {:?}",
                attached.placement, comment.text,
            )
            .unwrap();
        }
    }

    output
}

fn assert_borrowed<T>(value: &Spanned<'_, T>, source: &str) {
    let mut end = 0;

    for attached in &value.comments {
        let comment = &attached.comment;
        assert!(comment.span.start() >= end);
        assert_eq!(
            comment.span.slice(source).unwrap(),
            format!("#{}", comment.text)
        );
        assert!(std::ptr::eq(
            comment.text.as_ptr(),
            source[comment.span.start() + 1..].as_ptr()
        ));
        end = comment.span.end();
    }
}

#[derive(Debug, Deserialize)]
struct Config<'a> {
    #[serde(borrow)]
    one: Spanned<'a, u8>,
    #[serde(borrow)]
    nested: Spanned<'a, BTreeMap<String, Spanned<'a, u8>>>,
    #[serde(borrow)]
    last: Spanned<'a, u8>,
}

#[test]
fn parents_collect_comments_with_their_original_placement() {
    let source = r"# first
one: 1 # one

# detached

nested: # header
  # child
  two: 2 # two
  # nested tail
# last
last: 3
# root tail

# detached tail
";
    let value: Spanned<'_, Config<'_>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("one", &value.one.comments),
            ("nested", &value.nested.comments),
            ("nested.two", &value.nested["two"].comments),
            ("last", &value.last.comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Leading @1:1..1:8 " first"
      Inline @2:8..2:13 " one"
      Detached @4:1..4:11 " detached"
      Inline @6:9..6:17 " header"
      Leading @7:3..7:10 " child"
      Inline @8:10..8:15 " two"
      Trailing @9:3..9:16 " nested tail"
      Leading @10:1..10:7 " last"
      Trailing @12:1..12:12 " root tail"
      Detached @14:1..14:16 " detached tail"
    one:
      Leading @1:1..1:8 " first"
      Inline @2:8..2:13 " one"
    nested:
      Inline @6:9..6:17 " header"
      Leading @7:3..7:10 " child"
      Inline @8:10..8:15 " two"
      Trailing @9:3..9:16 " nested tail"
    nested.two:
      Leading @7:3..7:10 " child"
      Inline @8:10..8:15 " two"
    last:
      Leading @10:1..10:7 " last"
    "#);

    assert_borrowed(&value, source);

    let unwrapped_children: Spanned<'_, serde::de::IgnoredAny> = from_str(source).unwrap();
    assert_eq!(value.comments, unwrapped_children.comments);
    let nested: Spanned<'_, Spanned<'_, Config<'_>>> = from_str(source).unwrap();
    assert_eq!(nested.comments, nested.value.comments);
}

#[test]
fn sequence_headers_and_empty_items_own_their_comments() {
    let source = r"# first
- 1 # inline
- # empty
# third
- 3
# end
";
    let value: Spanned<'_, Vec<Spanned<'_, Option<u8>>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("[0]", &value[0].comments),
            ("[1]", &value[1].comments),
            ("[2]", &value[2].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Leading @1:1..1:8 " first"
      Inline @2:5..2:13 " inline"
      Inline @3:3..3:10 " empty"
      Leading @4:1..4:8 " third"
      Trailing @6:1..6:6 " end"
    [0]:
      Leading @1:1..1:8 " first"
      Inline @2:5..2:13 " inline"
    [1]:
      Inline @3:3..3:10 " empty"
    [2]:
      Leading @4:1..4:8 " third"
    "#);

    assert_borrowed(&value, source);

    let source = r"- # item header
  name: first # field
- name: second
";
    let value: Spanned<'_, Vec<Spanned<'_, Fields<'_>>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("[0]", &value[0].comments),
            ("[0].name", &value[0]["name"].comments),
            ("[1]", &value[1].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    [0]:
      Inline @1:3..1:16 " item header"
      Inline @2:15..2:22 " field"
    [0].name:
      Inline @2:15..2:22 " field"
    [1]:
      (none)
    "#);
}

#[test]
fn flow_delimiters_and_empty_containers_own_their_comments() {
    let source = r"[ # opening
 # first
 1, # one
 # second
 2,
 # tail
] # closing
";
    let value: Spanned<'_, Vec<Spanned<'_, u8>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("[0]", &value[0].comments),
            ("[1]", &value[1].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Inline @1:3..1:12 " opening"
      Leading @2:2..2:9 " first"
      Inline @3:5..3:10 " one"
      Leading @4:2..4:10 " second"
      Trailing @6:2..6:8 " tail"
      Inline @7:3..7:12 " closing"
    [0]:
      Leading @2:2..2:9 " first"
      Inline @3:5..3:10 " one"
    [1]:
      Leading @4:2..4:10 " second"
    "#);

    for source in ["[\n # empty\n]", "{\n # empty\n}"] {
        let value: Spanned<'_, serde::de::IgnoredAny> = from_str(source).unwrap();
        assert_eq!(comments(&value), [(" empty", Detached)]);
    }

    let source = "{empty: # null\n, next: 1 # number\n}";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, Option<u8>>>> = from_str(source).unwrap();
    assert_eq!(comments(&value["empty"]), [(" null", Inline)]);
    assert_eq!(comments(&value["next"]), [(" number", Inline)]);
}

#[test]
fn explicit_complex_keys_keep_key_specific_comments() {
    let source = r"# field
? [a, # child
   b] # key
: 1 # value
";
    let value: Spanned<'_, HashMap<ComplexKey<'_>, Spanned<'_, u8>>> = from_str(source).unwrap();
    let (key, number) = value.iter().next().unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("key", &key.comments),
            ("key[0]", &key[0].comments),
            ("value", &number.comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Leading @1:1..1:8 " field"
      Inline @2:7..2:14 " child"
      Inline @3:7..3:12 " key"
      Inline @4:5..4:12 " value"
    key:
      Inline @2:7..2:14 " child"
      Inline @3:7..3:12 " key"
    key[0]:
      Inline @2:7..2:14 " child"
    value:
      Leading @1:1..1:8 " field"
      Inline @4:5..4:12 " value"
    "#);
}

#[test]
fn aliases_collect_use_site_comments_without_importing_targets() {
    let source = r"definition: &items # definition
  - 1 # child
alias: *items # use
";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, Vec<Spanned<'_, u8>>>>> =
        from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("definition", &value["definition"].comments),
            ("alias", &value["alias"].comments),
            ("alias[0]", &value["alias"][0].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Inline @1:20..1:32 " definition"
      Inline @2:7..2:14 " child"
      Inline @3:15..3:20 " use"
    definition:
      Inline @1:20..1:32 " definition"
      Inline @2:7..2:14 " child"
    alias:
      Inline @3:15..3:20 " use"
    alias[0]:
      Inline @2:7..2:14 " child"
    "#);

    assert_borrowed(&value, source);
}

#[test]
fn properties_and_block_scalar_headers_belong_to_their_node() {
    let source = r"value: &value # anchor
  !!str # tag
  |- # scalar
    # scalar content
";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, String>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("value", &value["value"].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    root:
      Inline @1:15..1:23 " anchor"
      Inline @2:9..2:14 " tag"
      Inline @3:6..3:14 " scalar"
    value:
      Inline @1:15..1:23 " anchor"
      Inline @2:9..2:14 " tag"
      Inline @3:6..3:14 " scalar"
    "#);

    assert_eq!(value["value"].value, "# scalar content");

    let source = "&items # anchor\n- 1\n";
    let value: Spanned<'_, Vec<Spanned<'_, u8>>> = from_str(source).unwrap();
    assert_eq!(comments(&value), [(" anchor", Inline)]);
    assert_eq!(value[0].comments, []);
}

#[test]
fn document_markers_and_gaps_stay_with_the_correct_root() {
    let source = r"# preamble

%YAML 1.2 # directive
--- # start
1 # one
... # end
# gap
--- # second
2 # two
# suffix
";
    let stream = astral_yaml::parse(source).unwrap();
    let first: Spanned<'_, u8> = from_document(&stream, 0).unwrap();
    let second: Spanned<'_, u8> = from_document(&stream, 1).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("document 1", &first.comments),
            ("document 2", &second.comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @r#"
    document 1:
      Detached @1:1..1:11 " preamble"
      Inline @3:11..3:22 " directive"
      Inline @4:5..4:12 " start"
      Inline @5:3..5:8 " one"
      Inline @6:5..6:10 " end"
      Trailing @7:1..7:6 " gap"
    document 2:
      Inline @8:5..8:13 " second"
      Inline @9:3..9:8 " two"
      Trailing @10:1..10:9 " suffix"
    "#);

    assert_borrowed(&first, source);
    assert_borrowed(&second, source);

    let value: Spanned<'_, ()> = from_str("--- # empty\n").unwrap();
    assert_eq!(comments(&value), [(" empty", Inline)]);
    let stream = astral_yaml::parse("# only a comment\n").unwrap();
    assert_eq!(stream.documents(), []);
    assert_eq!(stream.comments()[0].text, " only a comment");
}

#[test]
fn unicode_crlf_and_original_encoding_spans_are_preserved() {
    for encoding in ["utf16", "utf32"] {
        let text = "\u{feff}name: café # 🌍  \r\n\r\n# footer\r\n";
        let bytes: Vec<u8> = if encoding == "utf16" {
            text.encode_utf16().flat_map(u16::to_le_bytes).collect()
        } else {
            text.chars()
                .flat_map(|c| u32::from(c).to_be_bytes())
                .collect()
        };
        let source = astral_yaml::Source::from_bytes(&bytes).unwrap();
        let stream = source.parse().unwrap();
        let value: Spanned<'_, BTreeMap<String, String>> = from_document(&stream, 0).unwrap();
        assert_eq!(comments(&value), [(" 🌍  ", Inline), (" footer", Detached)]);
        assert_borrowed(&value, source.text());

        let span = source
            .original_span(value.comments[0].comment.span)
            .unwrap();
        let expected: Vec<u8> = if encoding == "utf16" {
            "# 🌍  ".encode_utf16().flat_map(u16::to_le_bytes).collect()
        } else {
            "# 🌍  "
                .chars()
                .flat_map(|c| u32::from(c).to_be_bytes())
                .collect()
        };
        assert_eq!(span.slice_bytes(&bytes).unwrap(), expected);
    }
}

#[test]
fn owned_values_and_transparent_serialization_remain_supported() {
    fn parse_owned<T: DeserializeOwned>(source: String) -> T {
        let value = from_str(&source).unwrap();
        drop(source);
        value
    }

    let owned: BTreeMap<String, String> = parse_owned("name: example # comment".into());
    assert_eq!(owned["name"], "example");
    let mut wrapped: Spanned<'_, String> = from_str("example # comment").unwrap();
    let metadata = wrapped.comments.clone();
    wrapped.push('!');
    assert_eq!(wrapped.comments, metadata);
    assert_eq!(
        to_string(&wrapped).unwrap(),
        to_string(&wrapped.value).unwrap()
    );
    assert_eq!(wrapped.into_inner(), "example!");
}

#[test]
fn comments_at_scalar_boundaries_and_dedents_keep_their_scope() {
    let source = r"first: |
  text
# second
second: next
";
    let value: Spanned<'_, Fields<'_>> = from_str(source).unwrap();
    assert_eq!(value["first"].comments, []);
    assert_eq!(comments(&value["second"]), [(" second", Leading)]);

    let source = r"items:
  - 1
# root tail
";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, Vec<u8>>>> = from_str(source).unwrap();
    assert_eq!(value["items"].comments, []);
    assert_eq!(comments(&value), [(" root tail", Trailing)]);

    let value: Spanned<'_, u8> = from_str("# before\n\n42\n\n# after").unwrap();
    assert_eq!(
        comments(&value),
        [(" before", Detached), (" after", Detached)]
    );

    let value: Spanned<'_, u8> = from_str("# before\n42 # after").unwrap();
    assert_eq!(comments(&value), [(" before", Leading), (" after", Inline)]);

    let source = "\u{feff}# first\nfirst: text\n";
    let value: Spanned<'_, Fields<'_>> = from_str(source).unwrap();
    assert_eq!(comments(&value["first"]), [(" first", Leading)]);
}

#[test]
fn aggregation_does_not_expand_cyclic_aliases() {
    #[derive(Debug)]
    struct Opaque;

    impl<'de> Deserialize<'de> for Opaque {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Ok(Self)
        }
    }

    let source = "&self [*self # cycle\n]";
    let value: Spanned<'_, Opaque> = from_str(source).unwrap();
    assert_eq!(comments(&value), [(" cycle", Inline)]);
    assert_borrowed(&value, source);
    assert!(from_str::<Spanned<'_, serde::de::IgnoredAny>>(source).is_err());
}
