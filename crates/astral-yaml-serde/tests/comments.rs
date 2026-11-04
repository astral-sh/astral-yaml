//! YAML comments are presentation details. This API records their placement and ownership.
//! <https://yaml.org/spec/1.2.2/#3233-comments>
//! <https://yaml.org/spec/1.2.2/#66-comments>

use test_case::test_case;

use astral_yaml_dev::span_snapshot;
use astral_yaml_serde::{
    AttachedComment, CommentPlacement, Commented, from_document, from_str, to_string,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use CommentPlacement::{Detached, Inline, Leading, Trailing};

type Fields<'a> = BTreeMap<String, Commented<'a, String>>;
type ComplexKey<'a> = Commented<'a, Vec<Commented<'a, String>>>;

fn comments<'a, T>(value: &Commented<'a, T>) -> Vec<(&'a str, CommentPlacement)> {
    value
        .comments
        .iter()
        .map(|attached| (attached.comment.text, attached.placement))
        .collect()
}

fn comment_snapshot(source: &str, groups: &[(&str, &[AttachedComment<'_>])]) -> String {
    let mut annotations = BTreeMap::new();
    let mut empty = Vec::new();
    for (name, comments) in groups {
        if comments.is_empty() {
            empty.push(*name);
        }
        assert!(
            comments
                .windows(2)
                .all(|pair| { pair[0].comment.span.end() <= pair[1].comment.span.start() })
        );
        for attached in *comments {
            let comment = &attached.comment;
            assert_eq!(
                comment.span.slice(source).unwrap(),
                format!("#{}", comment.text)
            );
            let (_, labels) = annotations
                .entry((comment.span.start(), comment.span.end()))
                .or_insert_with(|| (comment.span, Vec::new()));
            labels.push(format!("{name}: {:?}", attached.placement));
        }
    }

    let mut output = span_snapshot(
        source,
        annotations
            .into_values()
            .map(|(span, labels)| (labels.join(", "), span)),
    );
    for name in empty {
        write!(output, "\n{name}: (none)").unwrap();
    }
    output
}

fn assert_borrowed<T>(value: &Commented<'_, T>, source: &str) {
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
    one: Commented<'a, u8>,
    #[serde(borrow)]
    nested: Commented<'a, BTreeMap<String, Commented<'a, u8>>>,
    #[serde(borrow)]
    last: Commented<'a, u8>,
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
    let value: Commented<'_, Config<'_>> = from_str(source).unwrap();

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

    insta::assert_snapshot!(snapshot, @"
       |
     1 | # first
       | ^^^^^^^ root: Leading, one: Leading
     2 | one: 1 # one
       |        ^^^^^ root: Inline, one: Inline
     3 |
     4 | # detached
       | ^^^^^^^^^^ root: Detached
     5 |
     6 | nested: # header
       |         ^^^^^^^^ root: Inline, nested: Inline
     7 |   # child
       |   ^^^^^^^ root: Leading, nested: Leading, nested.two: Leading
     8 |   two: 2 # two
       |          ^^^^^ root: Inline, nested: Inline, nested.two: Inline
     9 |   # nested tail
       |   ^^^^^^^^^^^^^ root: Trailing, nested: Trailing
    10 | # last
       | ^^^^^^ root: Leading, last: Leading
    11 | last: 3
    12 | # root tail
       | ^^^^^^^^^^^ root: Trailing
    13 |
    14 | # detached tail
       | ^^^^^^^^^^^^^^^ root: Detached
    ");

    assert_borrowed(&value, source);

    let unwrapped_children: Commented<'_, serde::de::IgnoredAny> = from_str(source).unwrap();
    assert_eq!(value.comments, unwrapped_children.comments);
    let nested: Commented<'_, Commented<'_, Config<'_>>> = from_str(source).unwrap();
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
    let value: Commented<'_, Vec<Commented<'_, Option<u8>>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("[0]", &value[0].comments),
            ("[1]", &value[1].comments),
            ("[2]", &value[2].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @"
      |
    1 | # first
      | ^^^^^^^ root: Leading, [0]: Leading
    2 | - 1 # inline
      |     ^^^^^^^^ root: Inline, [0]: Inline
    3 | - # empty
      |   ^^^^^^^ root: Inline, [1]: Inline
    4 | # third
      | ^^^^^^^ root: Leading, [2]: Leading
    5 | - 3
    6 | # end
      | ^^^^^ root: Trailing
    ");

    assert_borrowed(&value, source);

    let source = r"- # item header
  name: first # field
- name: second
";
    let value: Commented<'_, Vec<Commented<'_, Fields<'_>>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("[0]", &value[0].comments),
            ("[0].name", &value[0]["name"].comments),
            ("[1]", &value[1].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @"
      |
    1 | - # item header
      |   ^^^^^^^^^^^^^ [0]: Inline
    2 |   name: first # field
      |               ^^^^^^^ [0]: Inline, [0].name: Inline
    3 | - name: second
      |
    [1]: (none)
    ");
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
    let value: Commented<'_, Vec<Commented<'_, u8>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("[0]", &value[0].comments),
            ("[1]", &value[1].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @"
      |
    1 | [ # opening
      |   ^^^^^^^^^ root: Inline
    2 |  # first
      |  ^^^^^^^ root: Leading, [0]: Leading
    3 |  1, # one
      |     ^^^^^ root: Inline, [0]: Inline
    4 |  # second
      |  ^^^^^^^^ root: Leading, [1]: Leading
    5 |  2,
    6 |  # tail
      |  ^^^^^^ root: Trailing
    7 | ] # closing
      |   ^^^^^^^^^ root: Inline
    ");

    for source in ["[\n # empty\n]", "{\n # empty\n}"] {
        let value: Commented<'_, serde::de::IgnoredAny> = from_str(source).unwrap();
        assert_eq!(comments(&value), [(" empty", Detached)]);
    }

    let source = "{empty: # null\n, next: 1 # number\n}";
    let value: Commented<'_, BTreeMap<String, Commented<'_, Option<u8>>>> =
        from_str(source).unwrap();
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
    let value: Commented<'_, HashMap<ComplexKey<'_>, Commented<'_, u8>>> =
        from_str(source).unwrap();
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

    insta::assert_snapshot!(snapshot, @"
      |
    1 | # field
      | ^^^^^^^ root: Leading, value: Leading
    2 | ? [a, # child
      |       ^^^^^^^ root: Inline, key: Inline, key[0]: Inline
    3 |    b] # key
      |       ^^^^^ root: Inline, key: Inline
    4 | : 1 # value
      |     ^^^^^^^ root: Inline, value: Inline
    ");
}

#[test]
fn aliases_collect_use_site_comments_without_importing_targets() {
    let source = r"definition: &items # definition
  - 1 # child
alias: *items # use
";
    let value: Commented<'_, BTreeMap<String, Commented<'_, Vec<Commented<'_, u8>>>>> =
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

    insta::assert_snapshot!(snapshot, @"
      |
    1 | definition: &items # definition
      |                    ^^^^^^^^^^^^ root: Inline, definition: Inline
    2 |   - 1 # child
      |       ^^^^^^^ root: Inline, definition: Inline, alias[0]: Inline
    3 | alias: *items # use
      |               ^^^^^ root: Inline, alias: Inline
    ");

    assert_borrowed(&value, source);
}

#[test]
fn properties_and_block_scalar_headers_belong_to_their_node() {
    let source = r"value: &value # anchor
  !!str # tag
  |- # scalar
    # scalar content
";
    let value: Commented<'_, BTreeMap<String, Commented<'_, String>>> = from_str(source).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("root", &value.comments),
            ("value", &value["value"].comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @"
      |
    1 | value: &value # anchor
      |               ^^^^^^^^ root: Inline, value: Inline
    2 |   !!str # tag
      |         ^^^^^ root: Inline, value: Inline
    3 |   |- # scalar
      |      ^^^^^^^^ root: Inline, value: Inline
    4 |     # scalar content
      |
    ");

    // https://yaml.org/spec/1.2.2/#812-literal-style
    assert_eq!(value["value"].value, "# scalar content");

    let source = "&items # anchor\n- 1\n";
    let value: Commented<'_, Vec<Commented<'_, u8>>> = from_str(source).unwrap();
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
    let first: Commented<'_, u8> = from_document(&stream, 0).unwrap();
    let second: Commented<'_, u8> = from_document(&stream, 1).unwrap();

    let snapshot = comment_snapshot(
        source,
        &[
            ("document 1", &first.comments),
            ("document 2", &second.comments),
        ],
    );

    insta::assert_snapshot!(snapshot, @"
       |
     1 | # preamble
       | ^^^^^^^^^^ document 1: Detached
     2 |
     3 | %YAML 1.2 # directive
       |           ^^^^^^^^^^^ document 1: Inline
     4 | --- # start
       |     ^^^^^^^ document 1: Inline
     5 | 1 # one
       |   ^^^^^ document 1: Inline
     6 | ... # end
       |     ^^^^^ document 1: Inline
     7 | # gap
       | ^^^^^ document 1: Trailing
     8 | --- # second
       |     ^^^^^^^^ document 2: Inline
     9 | 2 # two
       |   ^^^^^ document 2: Inline
    10 | # suffix
       | ^^^^^^^^ document 2: Trailing
    ");

    assert_borrowed(&first, source);
    assert_borrowed(&second, source);

    let value: Commented<'_, ()> = from_str("--- # empty\n").unwrap();
    assert_eq!(comments(&value), [(" empty", Inline)]);
    // A stream may contain comments without a document.
    // https://yaml.org/spec/1.2.2/#92-streams
    let stream = astral_yaml::parse("# only a comment\n").unwrap();
    assert_eq!(stream.documents(), []);
    assert_eq!(stream.comments()[0].text, " only a comment");
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_case("utf16")]
#[test_case("utf32")]
fn unicode_crlf_and_original_encoding_spans_are_preserved(encoding: &str) {
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
    let value: Commented<'_, BTreeMap<String, String>> = from_document(&stream, 0).unwrap();
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

#[test]
fn owned_values_and_transparent_serialization_remain_supported() {
    fn parse_owned<T: DeserializeOwned>(source: String) -> T {
        let value = from_str(&source).unwrap();
        drop(source);
        value
    }

    let owned: BTreeMap<String, String> = parse_owned("name: example # comment".into());
    assert_eq!(owned["name"], "example");
    let mut wrapped: Commented<'_, String> = from_str("example # comment").unwrap();
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
    let value: Commented<'_, Fields<'_>> = from_str(source).unwrap();
    assert_eq!(value["first"].comments, []);
    assert_eq!(comments(&value["second"]), [(" second", Leading)]);

    let source = r"items:
  - 1
# root tail
";
    let value: Commented<'_, BTreeMap<String, Commented<'_, Vec<u8>>>> = from_str(source).unwrap();
    assert_eq!(value["items"].comments, []);
    assert_eq!(comments(&value), [(" root tail", Trailing)]);

    let value: Commented<'_, u8> = from_str("# before\n\n42\n\n# after").unwrap();
    assert_eq!(
        comments(&value),
        [(" before", Detached), (" after", Detached)]
    );

    let value: Commented<'_, u8> = from_str("# before\n42 # after").unwrap();
    assert_eq!(comments(&value), [(" before", Leading), (" after", Inline)]);

    let source = "\u{feff}# first\nfirst: text\n";
    let value: Commented<'_, Fields<'_>> = from_str(source).unwrap();
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

    // A sequence can contain itself.
    // https://yaml.org/spec/1.2.2/#3211-nodes
    let source = "&self [*self # cycle\n]";
    let value: Commented<'_, Opaque> = from_str(source).unwrap();
    assert_eq!(comments(&value), [(" cycle", Inline)]);
    assert_borrowed(&value, source);
    assert!(from_str::<Commented<'_, serde::de::IgnoredAny>>(source).is_err());
}

#[test]
fn spans_and_comments_compose_in_either_order() {
    use astral_yaml_serde::Spanned;

    let source = "# before\n42 # after";
    let first: Spanned<Commented<'_, u8>> = from_str(source).unwrap();
    let second: Commented<'_, Spanned<u8>> = from_str(source).unwrap();
    assert_eq!(first.span, second.span);
    assert_eq!(first.comments, second.comments);
    assert_eq!(first.value.value, second.value.value);

    let owned: Spanned<String> = {
        let source = String::from("text # gone");
        from_str(&source).unwrap()
    };
    assert_eq!(owned.into_inner(), "text");
}

#[test]
fn comments_require_the_yaml_deserializer() {
    let error = Commented::<u8>::deserialize(serde::de::value::U8Deserializer::<
        serde::de::value::Error,
    >::new(42))
    .unwrap_err();
    assert!(error.to_string().contains("a commented value"));
}

#[test]
fn comment_protocol_tracks_consumption() {
    use serde::de::{self, SeqAccess, Visitor};
    struct Protocol;
    impl<'de> Deserialize<'de> for Protocol {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            de.deserialize_newtype_struct("$astral_yaml::Commented", Self)
        }
    }
    impl<'de> Visitor<'de> for Protocol {
        type Value = Self;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("comment metadata")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self, A::Error> {
            assert_eq!(seq.size_hint(), Some(2));
            let value: u8 = seq.next_element()?.unwrap();
            assert_eq!(seq.size_hint(), Some(1));
            if value == 42 {
                seq.next_element::<de::IgnoredAny>()?.unwrap();
                assert_eq!(seq.size_hint(), Some(0));
                assert!(seq.next_element::<de::IgnoredAny>()?.is_none());
            }
            Ok(self)
        }
    }
    assert!(from_str::<Protocol>("42 # note").is_ok());
    assert!(from_str::<Protocol>("1 # note").is_err());
}

#[test]
fn document_markers_bound_child_comment_ownership() {
    let source = "# before directives\n%YAML 1.2\n# before marker\n--- # start\n# first field\nkey:\n  - old\n... # end\n  # after document\n";
    let mut snapshots = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = source.replace('\n', newline);
        let value: Commented<'_, BTreeMap<String, Commented<'_, Vec<Commented<'_, String>>>>> =
            from_str(&source).unwrap();
        snapshots.push(comment_snapshot(
            &source,
            &[
                ("root", &value.comments),
                ("key", &value["key"].comments),
                ("key[0]", &value["key"][0].comments),
            ],
        ));
    }
    insta::assert_snapshot!(snapshots.join("\n"), @"
      |
    1 | # before directives
      | ^^^^^^^^^^^^^^^^^^^ root: Leading
    2 | %YAML 1.2
    3 | # before marker
      | ^^^^^^^^^^^^^^^ root: Leading
    4 | --- # start
      |     ^^^^^^^ root: Inline
    5 | # first field
      | ^^^^^^^^^^^^^ root: Leading, key: Leading
    6 | key:
    7 |   - old
    8 | ... # end
      |     ^^^^^ root: Inline
    9 |   # after document
      |   ^^^^^^^^^^^^^^^^ root: Trailing
    key[0]: (none)
      |
    1 | # before directives
      | ^^^^^^^^^^^^^^^^^^^ root: Leading
    2 | %YAML 1.2
    3 | # before marker
      | ^^^^^^^^^^^^^^^ root: Leading
    4 | --- # start
      |     ^^^^^^^ root: Inline
    5 | # first field
      | ^^^^^^^^^^^^^ root: Leading, key: Leading
    6 | key:
    7 |   - old
    8 | ... # end
      |     ^^^^^ root: Inline
    9 |   # after document
      |   ^^^^^^^^^^^^^^^^ root: Trailing
    key[0]: (none)
      |
    1 | # before directives␍%YAML 1.2␍# before marker␍--- # start␍# first field␍key:␍  - old␍... # end␍  # after document␍
      | ^^^^^^^^^^^^^^^^^^^           ^^^^^^^^^^^^^^^     ^^^^^^^ ^^^^^^^^^^^^^                  ^^^^^   ^^^^^^^^^^^^^^^^ root: Trailing
      | |                             |                   |       |                              |
      | |                             |                   |       |                              root: Inline
      | |                             |                   |       root: Leading, key: Leading
      | |                             |                   root: Inline
      | |                             root: Leading
      | root: Leading
    key[0]: (none)
    ");
}
