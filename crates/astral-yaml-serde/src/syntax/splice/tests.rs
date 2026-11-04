use test_case::test_case;

use super::{Boundary, Line, PatchSet};
use crate::syntax::emit::{Ending, Fragment};
use astral_yaml::Span;
use astral_yaml_dev::{edit_diff, indoc};

#[test_case("new" => Ok("\u{feff}new\n\u{feff}".into()); "prefix and suffix lines were removed")]
#[test_case("# kept\n|2-\n  text\n# tail\n" => Ok("# kept\n\u{feff}|2-\n  text\n\u{feff}# tail\n".into()); "framing on both sides of a scalar")]
#[test_case("[a" => Err("expected ',' or ']'"); "malformed replacement")]
#[test_case("# only a comment" => Err("expected exactly one document"); "document was removed")]
#[test_case("one\n--- two" => Err("expected exactly one document"); "second document was introduced")]
fn patches_crossing_marks_need_a_document_boundary(
    replacement: &str,
) -> Result<String, &'static str> {
    // A document prefix permits a byte order mark before the content.
    // https://yaml.org/spec/1.2.2/#911-document-prefix
    let document =
        crate::syntax::DocumentSyntax::parse("# prefix\n\u{feff}old\n\u{feff}# suffix\n").unwrap();
    let source = &document.source;
    let span = crate::syntax::span(0, source.text.len());
    super::apply(source, vec![(span, replacement.into())], span).map_err(|error| error.message)
}

#[test_case("abc"; "retained mark")]
#[test_case("ab"; "separator before the mark")]
fn patch_limits_include_framing_after_the_replacement(replacement: &str) {
    let text = "a\n\u{feff}";
    let document = crate::syntax::DocumentSyntax::parse_with_options(
        text,
        astral_yaml::ParseOptions {
            max_input_bytes: text.len(),
            ..Default::default()
        },
    )
    .unwrap();
    let span = crate::syntax::span(0, text.len());
    let error = super::apply(&document.source, vec![(span, replacement.into())], span).unwrap_err();
    assert_eq!(error.message, "emitted YAML exceeds input byte limit");
}

#[test]
fn normalization_limit_includes_retained_framing() {
    let source = indoc! {"
        [a,b]
        # retained outside the body
    "};
    let document = crate::syntax::DocumentSyntax::parse_with_options(
        source,
        astral_yaml::ParseOptions {
            max_input_bytes: source.len(),
            ..astral_yaml::ParseOptions::default()
        },
    )
    .unwrap();
    insta::assert_debug_snapshot!(document.normalized_yaml(), @r#"
    Err(
        Error {
            span: Span {
                start: 0,
                end: 5,
            },
            message: "emitted YAML exceeds input byte limit",
        },
    )
    "#);
}

#[test]
fn lexical_endings_stay_inside_the_reserved_boundary() {
    // YAML block scalar chomping and comment termination:
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    // https://yaml.org/spec/1.2.2/#66-comments
    let source = indoc! {"
        value: old{padding}

        sibling: retained
    "}
    .replace("{padding}", "   ");
    let start = source.find("old").unwrap();
    let body = Span::new(start, start + 3).unwrap();
    let boundary = Boundary::new(&source, body, Line::at(&source, body.start()), false, None);
    let mut results = Vec::new();
    for (name, text, ending) in [
        ("token", "new", Ending::Token),
        ("comment", "new # comment", Ending::Comment),
        (
            "block scalar",
            indoc! {"
            |-
              new"},
            Ending::BlockScalar,
        ),
    ] {
        let replacement = boundary
            .complete(
                Fragment {
                    text: text.to_owned(),
                    ending,
                    last_block_child_indent: None,
                },
                1024,
            )
            .unwrap();
        assert!(boundary.reserved().start() <= replacement.span.start());
        assert!(replacement.span.end() <= boundary.reserved().end());
        let output = PatchSet::new(&source, vec![(replacement.span, replacement.text)], 1024)
            .unwrap()
            .apply();
        results.push(format!("{name}\n{}", edit_diff(&source, &output)));
    }
    insta::assert_snapshot!(results.join("\n"), @"
    token
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    +value: new   ⏎
     ⏎
     sibling: retained

    comment
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    +value: new # comment
     ⏎
     sibling: retained

    block scalar
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    -
    +value: |-
    +  new
     sibling: retained
    ");
}

#[test_case(
    indoc! {"
          old

        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1 @@
        -  old
        -
         ...
    ");
    "end marker"
)]
#[test_case(
    indoc! {"
        --- old

        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1,2 @@
        ---- old
        -
        +--- ⏎
         ...
    ");
    "start and end markers"
)]
fn erased_root_boundaries_retain_shared_marker_lines(source: &str) -> String {
    // Document markers delimit content and an explicit document may omit its root.
    // https://yaml.org/spec/1.2.2/#912-document-markers
    // https://yaml.org/spec/1.2.2/#914-explicit-documents
    let start = source.find("old").unwrap();
    let body = Span::new(start, start + 3).unwrap();
    let boundary = Boundary::new(source, body, Line::at(source, body.start()), true, None);
    let replacement = boundary
        .complete(
            Fragment {
                text: String::new(),
                ending: Ending::Token,
                last_block_child_indent: None,
            },
            1024,
        )
        .unwrap();
    assert!(boundary.reserved().start() <= replacement.span.start());
    assert!(replacement.span.end() <= boundary.reserved().end());
    let output = PatchSet::new(source, vec![(replacement.span, replacement.text)], 1024)
        .unwrap()
        .apply();
    edit_diff(source, &output)
}

#[test]
fn completing_a_boundary_checks_new_separation_against_the_limit() {
    let boundary = Boundary::new(
        "old\n",
        Span::new(0, 4).unwrap(),
        Line::at("old\n", 0),
        false,
        None,
    );
    let result = boundary.complete(
        Fragment {
            text: "new".to_owned(),
            ending: Ending::Token,
            last_block_child_indent: None,
        },
        3,
    );
    insta::assert_debug_snapshot!(result, @r#"
    Err(
        Error {
            span: Span {
                start: 0,
                end: 4,
            },
            message: "emitted YAML exceeds input byte limit",
        },
    )
    "#);
}

#[test]
fn patch_order_preserves_insertions_at_shared_boundaries() {
    let source = "αβγ";
    let patches = [
        (4, 6, "C"),
        (2, 4, "B"),
        (2, 2, "1"),
        (4, 4, "3"),
        (2, 2, "2"),
    ]
    .into_iter()
    .map(|(start, end, text)| (Span::new(start, end).unwrap(), text.to_owned()))
    .collect();
    let output = PatchSet::new(source, patches, 7).unwrap().apply();
    insta::assert_snapshot!(edit_diff(source, &output), @r"
    @@ -1 +1 @@
    -αβγ
    \ No newline at end of file
    +α12B3C
    \ No newline at end of file
    ");
}

#[test]
fn patch_application_checks_intervals_utf8_and_complete_output_size() {
    let mut results = Vec::new();
    for (patches, limit) in [
        (vec![], 4),
        (vec![(0, 2, "X")], 4),
        (vec![(2, 2, "1"), (2, 2, "2")], 6),
        (vec![(2, 4, "X"), (3, 4, "Y")], 10),
        (vec![(2, 5, "X")], 10),
        (vec![(0, 1, "X")], 10),
        (vec![(2, 2, "123")], 4),
        (vec![(0, 2, "123")], 4),
    ] {
        let patches = patches
            .into_iter()
            .map(|(start, end, replacement)| {
                (Span::new(start, end).unwrap(), replacement.to_owned())
            })
            .collect();
        results.push(PatchSet::new("éab", patches, limit).map(PatchSet::apply));
    }
    insta::assert_debug_snapshot!(results, @r#"
    [
        Ok(
            "éab",
        ),
        Ok(
            "Xab",
        ),
        Ok(
            "é12ab",
        ),
        Err(
            "overlapping or invalid source patches",
        ),
        Err(
            "overlapping or invalid source patches",
        ),
        Err(
            "overlapping or invalid source patches",
        ),
        Err(
            "emitted YAML exceeds input byte limit",
        ),
        Err(
            "emitted YAML exceeds input byte limit",
        ),
    ]
    "#);
}
