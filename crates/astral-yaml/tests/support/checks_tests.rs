use crate::checks::{event_tag, events, spans::validate_parts, validate_spans};
use astral_yaml::{Span, parse};

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
