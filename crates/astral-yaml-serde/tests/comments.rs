use astral_yaml_serde::{CommentPlacement, Spanned, from_document, from_str, to_string};
use serde::{Deserialize, de::DeserializeOwned};
use std::collections::{BTreeMap, HashMap};

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
    let source = "# first\none: 1 # one\n\n# detached\n\nnested: # header\n  # child\n  two: 2 # two\n  # nested tail\n# last\nlast: 3\n# root tail\n\n# detached tail\n";
    let value: Spanned<'_, Config<'_>> = from_str(source).unwrap();
    assert_eq!(
        comments(&value.one),
        [(" first", Leading), (" one", Inline)]
    );
    assert_eq!(
        comments(&value.nested["two"]),
        [(" child", Leading), (" two", Inline)]
    );
    assert_eq!(
        comments(&value.nested),
        [
            (" header", Inline),
            (" child", Leading),
            (" two", Inline),
            (" nested tail", Trailing),
        ]
    );
    assert_eq!(comments(&value.last), [(" last", Leading)]);
    assert_eq!(
        comments(&value),
        [
            (" first", Leading),
            (" one", Inline),
            (" detached", Detached),
            (" header", Inline),
            (" child", Leading),
            (" two", Inline),
            (" nested tail", Trailing),
            (" last", Leading),
            (" root tail", Trailing),
            (" detached tail", Detached),
        ]
    );
    assert_borrowed(&value, source);

    let unwrapped_children: Spanned<'_, serde::de::IgnoredAny> = from_str(source).unwrap();
    assert_eq!(value.comments, unwrapped_children.comments);
    let nested: Spanned<'_, Spanned<'_, Config<'_>>> = from_str(source).unwrap();
    assert_eq!(nested.comments, nested.value.comments);
}

#[test]
fn sequence_headers_and_empty_items_own_their_comments() {
    let source = "# first\n- 1 # inline\n- # empty\n# third\n- 3\n# end\n";
    let value: Spanned<'_, Vec<Spanned<'_, Option<u8>>>> = from_str(source).unwrap();
    assert_eq!(
        comments(&value[0]),
        [(" first", Leading), (" inline", Inline)]
    );
    assert_eq!(comments(&value[1]), [(" empty", Inline)]);
    assert_eq!(comments(&value[2]), [(" third", Leading)]);
    assert_eq!(value.comments.last().unwrap().placement, Trailing);
    assert_borrowed(&value, source);

    let source = "- # item header\n  name: first # field\n- name: second\n";
    let value: Spanned<'_, Vec<Spanned<'_, Fields<'_>>>> = from_str(source).unwrap();
    assert_eq!(
        comments(&value[0]),
        [(" item header", Inline), (" field", Inline)]
    );
    assert_eq!(comments(&value[0]["name"]), [(" field", Inline)]);
    assert!(value[1].comments.is_empty());
}

#[test]
fn flow_delimiters_and_empty_containers_own_their_comments() {
    let source = "[ # opening\n # first\n 1, # one\n # second\n 2,\n # tail\n] # closing\n";
    let value: Spanned<'_, Vec<Spanned<'_, u8>>> = from_str(source).unwrap();
    assert_eq!(comments(&value[0]), [(" first", Leading), (" one", Inline)]);
    assert_eq!(comments(&value[1]), [(" second", Leading)]);
    assert_eq!(
        comments(&value),
        [
            (" opening", Inline),
            (" first", Leading),
            (" one", Inline),
            (" second", Leading),
            (" tail", Trailing),
            (" closing", Inline),
        ]
    );

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
    let source = "# field\n? [a, # child\n   b] # key\n: 1 # value\n";
    let value: Spanned<'_, HashMap<ComplexKey<'_>, Spanned<'_, u8>>> = from_str(source).unwrap();
    let (key, number) = value.iter().next().unwrap();
    assert_eq!(comments(key), [(" child", Inline), (" key", Inline)]);
    assert_eq!(comments(&key[0]), [(" child", Inline)]);
    assert_eq!(comments(number), [(" field", Leading), (" value", Inline)]);
    assert_eq!(value.comments.len(), 4);
}

#[test]
fn aliases_collect_use_site_comments_without_importing_targets() {
    let source = "definition: &items # definition\n  - 1 # child\nalias: *items # use\n";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, Vec<Spanned<'_, u8>>>>> =
        from_str(source).unwrap();
    assert_eq!(
        comments(&value["definition"]),
        [(" definition", Inline), (" child", Inline)]
    );
    assert_eq!(comments(&value["alias"]), [(" use", Inline)]);
    assert_eq!(comments(&value["alias"][0]), [(" child", Inline)]);
    assert_eq!(
        comments(&value),
        [
            (" definition", Inline),
            (" child", Inline),
            (" use", Inline)
        ]
    );
    assert_borrowed(&value, source);
}

#[test]
fn properties_and_block_scalar_headers_belong_to_their_node() {
    let source = "value: &value # anchor\n  !!str # tag\n  |- # scalar\n    # scalar content\n";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, String>>> = from_str(source).unwrap();
    assert_eq!(
        comments(&value["value"]),
        [(" anchor", Inline), (" tag", Inline), (" scalar", Inline)]
    );
    assert_eq!(value["value"].value, "# scalar content");

    let source = "&items # anchor\n- 1\n";
    let value: Spanned<'_, Vec<Spanned<'_, u8>>> = from_str(source).unwrap();
    assert_eq!(comments(&value), [(" anchor", Inline)]);
    assert!(value[0].comments.is_empty());
}

#[test]
fn document_markers_and_gaps_stay_with_the_correct_root() {
    let source = "# preamble\n\n%YAML 1.2 # directive\n--- # start\n1 # one\n... # end\n# gap\n--- # second\n2 # two\n# suffix\n";
    let stream = astral_yaml::parse(source).unwrap();
    let first: Spanned<'_, u8> = from_document(&stream, 0).unwrap();
    let second: Spanned<'_, u8> = from_document(&stream, 1).unwrap();
    assert_eq!(
        comments(&first),
        [
            (" preamble", Detached),
            (" directive", Inline),
            (" start", Inline),
            (" one", Inline),
            (" end", Inline),
            (" gap", Trailing)
        ]
    );
    assert_eq!(
        comments(&second),
        [(" second", Inline), (" two", Inline), (" suffix", Trailing)]
    );
    assert_borrowed(&first, source);
    assert_borrowed(&second, source);

    let value: Spanned<'_, ()> = from_str("--- # empty\n").unwrap();
    assert_eq!(comments(&value), [(" empty", Inline)]);
    let stream = astral_yaml::parse("# only a comment\n").unwrap();
    assert!(stream.documents().is_empty());
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
    let source = "first: |\n  text\n# second\nsecond: next\n";
    let value: Spanned<'_, Fields<'_>> = from_str(source).unwrap();
    assert!(value["first"].comments.is_empty());
    assert_eq!(comments(&value["second"]), [(" second", Leading)]);

    let source = "items:\n  - 1\n# root tail\n";
    let value: Spanned<'_, BTreeMap<String, Spanned<'_, Vec<u8>>>> = from_str(source).unwrap();
    assert!(value["items"].comments.is_empty());
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
