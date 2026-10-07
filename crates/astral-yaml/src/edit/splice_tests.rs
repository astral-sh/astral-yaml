use super::{Boundary, PatchSet};
use crate::{
    Span,
    edit::emit::{Ending, Fragment},
};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn normalization_limit_includes_retained_framing() {
    let source = indoc! {"
        [a,b]
        # retained outside the body
    "};
    let document = crate::edit::EditableDocument::parse_with_options(
        source,
        crate::ParseOptions {
            max_input_bytes: source.len(),
            ..crate::ParseOptions::default()
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
    let boundary = Boundary::new(&source, body, false, None);
    let mut results = Vec::new();
    for (name, text, ending) in [
        ("token", "new", Ending::Token),
        ("comment", "new # comment", Ending::Comment),
        ("block scalar", "|-\n  new", Ending::BlockScalar),
    ] {
        let replacement = boundary
            .complete(
                Fragment {
                    text: text.to_owned(),
                    ending,
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
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    +value: new   ⏎
     ⏎
     sibling: retained

    comment
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    +value: new # comment
     ⏎
     sibling: retained

    block scalar
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -value: old   ⏎
    -
    +value: |-
    +  new
     sibling: retained
    ");
}

#[test]
fn erased_root_boundaries_retain_shared_marker_lines() {
    let mut results = Vec::new();
    for source in [
        indoc! {"
              old

            ...
        "},
        indoc! {"
            --- old

            ...
        "},
    ] {
        let start = source.find("old").unwrap();
        let body = Span::new(start, start + 3).unwrap();
        let boundary = Boundary::new(source, body, true, None);
        let replacement = boundary
            .complete(
                Fragment {
                    text: String::new(),
                    ending: Ending::Token,
                },
                1024,
            )
            .unwrap();
        assert!(boundary.reserved().start() <= replacement.span.start());
        assert!(replacement.span.end() <= boundary.reserved().end());
        let output = PatchSet::new(source, vec![(replacement.span, replacement.text)], 1024)
            .unwrap()
            .apply();
        results.push(edit_diff(source, &output));
    }
    insta::assert_snapshot!(results.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1 @@
    -  old
    -
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    ---- old
    -
    +--- ⏎
     ...
    ");
}

#[test]
fn completing_a_boundary_checks_new_separation_against_the_limit() {
    let boundary = Boundary::new("old\n", Span::new(0, 4).unwrap(), false, None);
    let result = boundary.complete(
        Fragment {
            text: "new".to_owned(),
            ending: Ending::Token,
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
    --- before.yaml
    +++ after.yaml
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
