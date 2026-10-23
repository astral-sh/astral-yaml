use test_case::test_matrix;

use astral_yaml::{Encoding, Source, Span};

fn encode(text: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => text.as_bytes().to_vec(),
        Encoding::Utf16Be => text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
        Encoding::Utf16Le => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        Encoding::Utf32Be => text
            .chars()
            .flat_map(|c| u32::from(c).to_be_bytes())
            .collect(),
        Encoding::Utf32Le => text
            .chars()
            .flat_map(|c| u32::from(c).to_le_bytes())
            .collect(),
    }
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_matrix(
    [Encoding::Utf8, Encoding::Utf16Be, Encoding::Utf16Le, Encoding::Utf32Be, Encoding::Utf32Le],
    ["", "\u{feff}"]
)]
fn encodings_and_original_spans(encoding: Encoding, bom: &str) {
    let text = format!("{bom}---\na: '🌍 é' # café\n");
    let bytes = encode(&text, encoding);
    let source: Source<'_> = bytes.as_slice().try_into().unwrap();
    assert_eq!(source.text(), text);
    assert_eq!(source.encoding(), encoding);
    assert_eq!(source.bytes(), bytes);

    let stream = source.parse().unwrap();

    let spans = stream
        .nodes()
        .iter()
        .map(|n| n.span)
        .chain(stream.comments().iter().map(|c| c.span))
        .chain(stream.byte_order_marks().iter().copied());
    for span in spans {
        let original = source.original_span(span).unwrap();
        assert_eq!(
            original.slice_bytes(&bytes).unwrap(),
            encode(span.slice(&text).unwrap(), encoding)
        );
    }

    let inside = text.find('🌍').unwrap() + 1;
    assert!(
        source
            .original_span(Span::new(inside, inside).unwrap())
            .is_none()
    );
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test]
fn invalid_encodings_and_limits() {
    for bytes in [
        &[0xff][..],
        &[0xef, 0xbb],
        &[0xff, 0xfe, 1],
        &[0xfe, 0xff, 0xd8, 0],
        &[0xfe, 0xff, 0xd8, 0, 0, 1],
        &[0xff, 0xfe, 0, 0xdc],
        &[0, 0, 0xfe, 0xff, 0],
        &[0xff, 0xfe, 0, 0, 0, 0xd8, 0, 0],
        &[0, 0, 0xfe, 0xff, 0, 0x11, 0, 0],
    ] {
        let error = Source::try_from(bytes).unwrap_err();
        assert!(error.span.slice_bytes(bytes).is_some());
    }

    assert!(Source::with_limit(b"abc", 2).is_err());
    assert!(Source::with_limit(&[0xfe, 0xff], 2).is_err());

    assert_eq!(Source::from_bytes(b"").unwrap().text(), "");
    assert!(Source::from_bytes(b"[").unwrap().parse().is_err());
}
