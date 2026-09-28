use crate::{Value, events, events::event_tag, spans::validate_parts, validate_spans};
use astral_yaml::{Span, parse};

#[test]
fn test_values_preserve_scalar_and_collection_types() {
    for (source, expected) in [
        ("null", Value::Null),
        ("true", Value::Bool(true)),
        ("-42", Value::Integer(-42)),
        ("18446744073709551615", Value::Unsigned(u64::MAX)),
        ("1.5", Value::Float(1.5)),
        (".nan", Value::Float(f64::NAN)),
        (".inf", Value::Float(f64::INFINITY)),
        ("hello", Value::String("hello".to_owned())),
        ("[null]", Value::Sequence(vec![Value::Null])),
        (
            "{[true]: false}",
            Value::Mapping(vec![(
                Value::Sequence(vec![Value::Bool(true)]),
                Value::Bool(false),
            )]),
        ),
    ] {
        let value = astral_yaml_serde::from_str::<Value>(source).unwrap();
        assert_eq!(value, expected, "{source}");

        let yaml = astral_yaml_serde::to_string(&value).unwrap();
        assert_eq!(
            astral_yaml_serde::from_str::<Value>(&yaml).unwrap(),
            expected
        );
    }
}

#[test]
fn test_value_equality_ignores_mapping_order_but_preserves_types_and_entries() {
    for (left, right, equal) in [
        ("{a: 1, b: 2}", "{b: 2, a: 1}", true),
        ("{a: 1}", "{b: 1}", false),
        ("{a: 1}", "{a: 2}", false),
        ("{a: 1}", "{}", false),
        ("{[true]: 1}", "{[1]: 1}", false),
        ("[1, 2]", "[2, 1]", false),
        ("1", "1.0", false),
        (".nan", ".nan", true),
        ("[.nan]", "[.nan]", true),
        ("{key: .nan}", "{key: .nan}", true),
        ("{.nan: value}", "{.nan: value}", true),
        (".inf", "-.inf", false),
    ] {
        let left = astral_yaml_serde::from_str::<Value>(left).unwrap();
        let right = astral_yaml_serde::from_str::<Value>(right).unwrap();
        assert_eq!(left == right, equal, "{left:?} == {right:?}");
    }

    assert_eq!(Value::Integer(42), Value::Unsigned(42));
    assert_eq!(Value::Unsigned(42), Value::Integer(42));
    assert_ne!(Value::Integer(-1), Value::Unsigned(u64::MAX));

    let entry = (Value::Null, Value::Bool(true));
    assert_ne!(
        Value::Mapping(vec![entry.clone(), entry.clone()]),
        Value::Mapping(vec![entry, (Value::Null, Value::Bool(false))]),
    );
}

#[test]
fn nan_values_compare_equal_and_roundtrip() {
    for bits in [
        f64::NAN.to_bits(),
        (-f64::NAN).to_bits(),
        0x7ff0_0000_0000_0001,
        0x7ff8_0000_0000_0042,
    ] {
        assert_eq!(Value::Float(f64::from_bits(bits)), Value::Float(f64::NAN));
    }

    for source in [
        ".nan",
        "[.nan]",
        "{key: .nan}",
        "{.nan: value}",
        "{[.nan]: {nested: [.nan]}}",
    ] {
        let value = astral_yaml_serde::from_str::<Value>(source).unwrap();
        assert_eq!(value, value.clone());

        let yaml = astral_yaml_serde::to_string(&value).unwrap();
        let decoded = astral_yaml_serde::from_str::<Value>(&yaml).unwrap();
        assert_eq!(value, decoded);
    }

    for (left, right) in [
        (".nan", "1.0"),
        ("[.nan]", "[.inf]"),
        ("{.nan: 1}", "{.nan: 2}"),
        ("{key: .nan}", "{other: .nan}"),
    ] {
        let left = astral_yaml_serde::from_str::<Value>(left).unwrap();
        let right = astral_yaml_serde::from_str::<Value>(right).unwrap();
        assert_ne!(left, right);
    }
}

#[test]
fn event_escaping() {
    let stream = parse("\"\\0\\b\\t\\r\\n\\\\\"").unwrap();
    assert!(events(&stream).contains("=VAL \"\\0\\b\\t\\r\\n\\\\\n"));
    assert_eq!(event_tag("!%FF"), "!%FF");
    assert_eq!(event_tag("!%C3%A9"), "!é");
}

#[test]
fn validator_rejects_bad_ranges_and_references() {
    for source in ["x", "[a]", "{a: b}", "[&a x, *a]"] {
        let stream = parse(source).unwrap();
        validate_spans(&stream).unwrap();
        let error = validate_parts(
            source,
            stream.nodes(),
            stream.documents(),
            &[],
            &[],
            &[],
            &|_| None,
        )
        .unwrap_err();
        assert!(error.contains("ID"));
    }

    let stream = parse("{a: b}").unwrap();
    let mut value = None;

    for node in stream.nodes() {
        if let astral_yaml::NodeKind::Mapping { entries, .. } = &node.kind {
            value = Some(entries[0].value);
        }
    }

    let value = value.unwrap();
    assert_eq!(
        validate_parts(stream.source(), stream.nodes(), &[], &[], &[], &[], &|id| {
            if id == value { None } else { stream.node(id) }
        })
        .unwrap_err(),
        "invalid value ID"
    );

    let stream = parse("[&a x, *a]").unwrap();
    let alias = stream.nodes().last().unwrap();
    assert_eq!(
        validate_parts(
            stream.source(),
            std::slice::from_ref(alias),
            &[],
            &[],
            &[],
            &[],
            &|_| None
        )
        .unwrap_err(),
        "invalid alias ID"
    );

    let stream = parse("'é'").unwrap();
    let lookup = &|id| stream.node(id);
    validate_parts(
        stream.source(),
        stream.nodes(),
        stream.documents(),
        &[],
        &[],
        &[],
        lookup,
    )
    .unwrap();

    let mut nodes = stream.nodes().to_vec();
    nodes[0].span = Span::new(0, 10).unwrap();
    assert!(
        validate_parts(stream.source(), &nodes, &[], &[], &[], &[], lookup)
            .unwrap_err()
            .contains("invalid span")
    );

    nodes[0].span = Span::new(0, 1).unwrap();
    assert!(
        validate_parts(stream.source(), &nodes, &[], &[], &[], &[], lookup)
            .unwrap_err()
            .contains("outside parent")
    );
}

#[test]
fn validator_checks_comments_and_byte_order_marks() {
    let stream = parse("\u{feff}# one\n# two\n%YAML 1.2\n---\nx\n...\n").unwrap();
    validate_spans(&stream).unwrap();

    let check = |boms: &[Span], comments: &[astral_yaml::Comment<'_>]| {
        validate_parts(
            stream.source(),
            stream.nodes(),
            stream.documents(),
            boms,
            stream.document_end_markers(),
            comments,
            &|id| stream.node(id),
        )
    };

    let bom = stream.byte_order_marks()[0];
    assert!(check(&[bom, bom], &[]).is_err());
    assert!(check(&[Span::new(3, 4).unwrap()], &[]).is_err());

    let mut comments = stream.comments().to_vec();
    comments.reverse();
    assert!(check(&[], &comments).is_err());

    comments[0].text = "different text";
    assert!(check(&[], &comments[..1]).is_err());
}
