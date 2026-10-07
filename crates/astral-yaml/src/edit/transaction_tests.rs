use crate::CollectionStyle;
use crate::edit::{
    draft::{Candidate, same_style},
    emit,
    framing::RootLayout,
    plan::Plan,
    view::Syntax,
};
use astral_yaml_dev::{edit_diff, indoc};

use super::{EditableDocument, Editor, Layout, NodeKind, Source, Span};

fn plan<'a>(
    editor: &'a Editor<'_>,
    source: &'a Source,
    candidate: &'a Candidate,
    rendering: Syntax<'a, Option<Span>>,
) -> Plan<'a> {
    super::plan(
        source,
        candidate,
        rendering,
        &editor.rewrites,
        editor.root_comments_changed,
    )
}

#[test]
fn a_rendering_copy_cannot_redefine_the_committed_edit() {
    let source = indoc! {"
        value: old
    "};
    let document = EditableDocument::parse(source).unwrap();
    let mut editor = Editor::new(&document);
    let value = editor.mapping_entries(&editor.root()).unwrap()[0].1.clone();
    editor.replace(&value, "requested").unwrap();

    // A broken planning or rendering pass must not change the draft against
    // which commit is checked. Deliberately corrupt only the rendering copy.
    let candidate = editor.draft.clone().freeze();
    let mut planned = Syntax::from(&candidate).nodes.to_vec();
    planned[value.id.0].kind = NodeKind::Scalar {
        value: "incorrect".to_owned(),
        style: crate::ScalarStyle::Plain,
    };
    let planned = Syntax {
        nodes: &planned,
        root: document.root,
    };
    let source_index = Source::from(&document);
    let output = plan(&editor, &source_index, &candidate, planned)
        .render(&Layout::new(planned))
        .unwrap();
    let error = candidate.commit(&output).unwrap_err();
    insta::assert_snapshot!(
        format!("{}\nCommit: {}", edit_diff(source, &output), error.message),
        @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -value: old
    +value: incorrect

    Commit: emission changed YAML content or alias identity
    ");
}

#[test]
fn overlapping_rewrite_regions_expand_to_their_common_ancestor() {
    let mut document = EditableDocument::parse("[[a, b], c]").unwrap();
    let first = crate::NodeId(
        document
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "a"))
            .unwrap(),
    );
    let last = crate::NodeId(
        document
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "c"))
            .unwrap(),
    );
    // The patch planner must coalesce overlapping retained ranges, including
    // ranges expanded by trivia ownership, before slicing the original source.
    document.nodes[first.0].span = Span::new(2, 10).unwrap();
    let mut editor = Editor::new(&document);
    editor.rewrites.extend([first, last]);
    let candidate = editor.draft.clone().freeze();
    let syntax = Layout::new(&candidate);
    let source_index = Source::from(&document);
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap(),
        [(document.root, document.nodes[document.root.0].span)]
    );
}

#[test]
fn empty_transactions_do_not_require_rewrite_work() {
    let document = EditableDocument::parse("one").unwrap();
    let editor = Editor::new(&document);
    let candidate = editor.draft.clone().freeze();
    candidate.work().set(0);
    let syntax = Layout::new(&candidate);
    let source_index = Source::from(&document);
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap(),
        []
    );
}

#[test]
fn outer_comment_plans_obey_work_and_output_limits() {
    let source = indoc! {r"
                # old
        value
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document.options.max_input_bytes = source.len();
    let mut editor = Editor::new(&document);
    for index in 0..3 {
        editor
            .insert_comment(
                &editor.root(),
                index,
                " new",
                crate::comments::CommentPlacement::Leading,
            )
            .unwrap();
    }
    let region = document.nodes[document.root.0].span;
    let candidate = editor.draft.clone().freeze();
    let syntax = Layout::new(&candidate);
    let source_index = Source::from(&document);
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .root_comments(Some(region), &layout)
            .unwrap_err()
            .message,
        "emitted YAML exceeds input byte limit"
    );
    candidate.work().set(0);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .root_comments(Some(region), &layout)
            .unwrap_err()
            .message,
        "editing resource limit exceeded"
    );
}

#[test]
fn requested_style_comparison_checks_kind_and_collection_contents() {
    for (left, right, expected) in [
        ("one", "one", true),
        ("one", "'one'", false),
        ("one", "[one]", false),
        ("[]", "[]", true),
        ("[one]", "- one", false),
        ("{}", "{}", true),
        ("{one: two}", "one: two", false),
    ] {
        let left = EditableDocument::parse(left).unwrap();
        let right = EditableDocument::parse(right).unwrap();
        assert_eq!(
            same_style(
                &left.nodes[left.root.0].kind,
                &right.nodes[right.root.0].kind
            ),
            expected
        );
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn collection_rewrites_include_their_comment_context() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            outer:
            # detached
              key: old
        "},
        indoc! {r"
            outer:
              # detached
                key: old
        "},
        indoc! {r"
            outer:
            # leading
              {key: old}
        "},
        indoc! {r"
            outer:
            # leading
              [old]
        "},
        indoc! {r"
            outer:
              [old] # inline
        "},
        indoc! {r"
            outer:
            # detached
              - old
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let target = editor.mapping_entries(&editor.root())?[0].1.clone();
                editor.set_collection_style(&target, CollectionStyle::Flow)?;
                match editor.node(&target)?.kind() {
                    NodeKind::Sequence { .. } => editor.insert_item(&target, 1, "new"),
                    _ => editor.insert_entry(&target, 1, "added", "new"),
                }
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  key: old
    +  # detached
    +
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
       # detached
    -    key: old
    +
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:
    -# leading
    -  {key: old}
    +outer: {
    +  # leading
    +  key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:
    -# leading
    -  [old]
    +outer: [
    +  # leading
    +  old, new]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -outer:
    -  [old] # inline
    +outer: [old, new] # inline

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  - old
    +  # detached
    +
    +  [old, new]
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
        results.push(crate::edit::plan::apply_patches("éab", patches, limit));
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

#[test]
fn nested_scalar_suffixes_expand_the_rewrite_to_its_boundary() {
    let document = EditableDocument::parse(indoc! {r"
        outer:
          value: old

        # detached
    "})
    .unwrap();
    let mut editor = Editor::new(&document);
    let inner = editor.mapping_entries(&editor.root()).unwrap()[0].1.clone();
    let value = editor.mapping_entries(&inner).unwrap()[0].1.clone();
    editor
        .replace(
            &value,
            indoc! {r"
                |+
                  new

            "},
        )
        .unwrap();
    let candidate = editor.draft.clone().freeze();
    let syntax = Layout::new(&candidate);
    let source_index = Source::from(&document);
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap(),
        [(document.root, emit::region(&document, document.root, true))]
    );
}

#[test]
fn enclosing_trailing_comments_require_a_nested_closing_boundary() {
    let document = EditableDocument::parse(indoc! {r"
        outer:
          value: old
        # trailing
    "})
    .unwrap();
    let mut editor = Editor::new(&document);
    let inner = editor.mapping_entries(&editor.root()).unwrap()[0].1.clone();
    editor
        .insert_comment(
            &inner,
            0,
            " detached",
            crate::comments::CommentPlacement::Detached,
        )
        .unwrap();
    let candidate = editor.draft.clone().freeze();
    let syntax = Layout::new(&candidate);
    assert!(syntax.flow(inner.id));
    editor.finish().unwrap();
}

#[test]
fn indented_root_suffixes_share_the_body_rewrite_context() {
    let document = EditableDocument::parse(indoc! {r"
        outer: {value: old}

           # root
    "})
    .unwrap();
    let mut editor = Editor::new(&document);
    let inner = editor.mapping_entries(&editor.root()).unwrap()[0].1.clone();
    let value = editor.mapping_entries(&inner).unwrap()[0].1.clone();
    editor.replace(&value, "new").unwrap();
    let candidate = editor.draft.clone().freeze();
    let syntax = Layout::new(&candidate);
    let source_index = Source::from(&document);
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&editor, &source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap(),
        [(document.root, emit::region(&document, document.root, true))]
    );
    editor.finish().unwrap();
}
