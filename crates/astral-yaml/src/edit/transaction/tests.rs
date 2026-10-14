use test_case::test_case;

use crate::CollectionStyle;
use crate::edit::{
    draft::{Candidate, same_style},
    framing::RootLayout,
    plan::{Changes, Plan},
    source::Source,
    splice,
    view::Syntax,
};
use astral_yaml_dev::{edit_diff, indoc};

use super::{EditableDocument, Editor, Layout, NodeKind, Span};

fn plan<'a>(
    source: &'a Source,
    candidate: &'a Candidate,
    rendering: Syntax<'a, Option<Span>>,
) -> Plan<'a> {
    let changes = Changes::between(source, candidate, &Layout::new(candidate));
    Plan {
        original: source,
        candidate: rendering,
        parents: candidate.parents(),
        changes,
        work: candidate.work(),
    }
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
    let candidate = editor.draft.clone().freeze().unwrap();
    let mut planned = Syntax::from(&candidate).nodes.to_vec();
    planned[value.id.0].kind = NodeKind::Scalar {
        value: "incorrect".to_owned(),
        style: crate::ScalarStyle::Plain,
    };
    let planned = Syntax {
        nodes: &planned,
        root: document.source.root,
    };
    let source_index = document.source.clone();
    let output = plan(&source_index, &candidate, planned)
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
            .source
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "a"))
            .unwrap(),
    );
    let last = crate::NodeId(
        document
            .source
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "c"))
            .unwrap(),
    );
    // The patch planner must coalesce overlapping retained ranges, including
    // ranges expanded by trivia ownership, before slicing the original source.
    document.source.nodes[first.0].span = Span::new(2, 10).unwrap();
    let mut editor = Editor::new(&document);
    editor.replace(&editor.handle(first), "new_a").unwrap();
    editor.replace(&editor.handle(last), "new_c").unwrap();
    let candidate = editor.draft.clone().freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source_index = document.source.clone();
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            document.source.nodes[document.source.root.0].span
        )]
    );
}

#[test]
fn empty_transactions_do_not_require_rewrite_work() {
    let document = EditableDocument::parse("one").unwrap();
    let editor = Editor::new(&document);
    let candidate = editor.draft.clone().freeze().unwrap();
    candidate.work().set(0);
    let syntax = Layout::new(&candidate);
    let source_index = document.source.clone();
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert!(
        plan(&source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn outer_comment_plans_obey_work_and_output_limits() {
    let source = indoc! {r"
                # old
        value
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document.source.options.max_input_bytes = source.len();
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
    let region = document.source.nodes[document.source.root.0].span;
    let candidate = editor.draft.clone().freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source_index = document.source.clone();
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&source_index, &candidate, (&candidate).into())
            .root_comments(Some(region), &layout)
            .unwrap_err()
            .message,
        "emitted YAML exceeds input byte limit"
    );
    candidate.work().set(0);
    assert_eq!(
        plan(&source_index, &candidate, (&candidate).into())
            .root_comments(Some(region), &layout)
            .unwrap_err()
            .message,
        "editing resource limit exceeded"
    );
}

#[test_case("one", "one", true)]
#[test_case("one", "'one'", false)]
#[test_case("one", "[one]", false)]
#[test_case("[]", "[]", true)]
#[test_case("[one]", "- one", false)]
#[test_case("{}", "{}", true)]
#[test_case("{one: two}", "one: two", false)]
fn requested_style_comparison_checks_kind_and_collection_contents(
    left: &str,
    right: &str,
    expected: bool,
) {
    let left = EditableDocument::parse(left).unwrap();
    let right = EditableDocument::parse(right).unwrap();
    assert_eq!(
        same_style(
            &left.source.nodes[left.source.root.0].kind,
            &right.source.nodes[right.source.root.0].kind
        ),
        expected
    );
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
                    NodeKind::Sequence { .. } => editor.insert_item(&target, 1, "new").map(|_| ()),
                    _ => editor.insert_entry(&target, 1, "added", "new").map(|_| ()),
                }
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
    -# detached
    +  # detached
     ⏎
    -  key: old
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
       # detached
     ⏎
    -    key: old
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -outer:
     # leading
    -  {key: old}
    +outer: {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -outer:
     # leading
    -  [old]
    +outer: [old, new]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -outer:
    -  [old] # inline
    +outer: [old, new] # inline

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
    -# detached
    +  # detached
     ⏎
    -  - old
    +  [old, new]
    ");
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
    let candidate = editor.draft.clone().freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source_index = document.source.clone();
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            splice::region(&document, document.source.root, true)
        )]
    );
}

#[test]
fn enclosing_trailing_comments_require_a_nested_closing_boundary() {
    let mut results = Vec::new();
    for source in [
        indoc! {r"
            outer:
              value: old
            # trailing
        "},
        indoc! {r"
            outer:
              value: |-
                old
            # trailing
        "},
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let mut editor = Editor::new(&document);
        let inner = editor.mapping_entries(&editor.root()).unwrap()[0].1.clone();
        let value = editor.mapping_entries(&inner).unwrap()[0].1.clone();
        let before = editor.draft.clone().freeze().unwrap();
        let multiline_before =
            Layout::new(&before).multiline(value.id, editor.node(&value).unwrap().kind());
        editor
            .insert_comment(
                &inner,
                0,
                " detached",
                crate::comments::CommentPlacement::Detached,
            )
            .unwrap();
        let candidate = editor.draft.clone().freeze().unwrap();
        let layout = Layout::new(&candidate);
        let multiline_after = layout.multiline(value.id, editor.node(&value).unwrap().kind());
        let edited = editor.finish().unwrap();
        results.push(format!(
            "Scalar multiline: {multiline_before} -> {multiline_after}\n{}",
            edit_diff(source, edited.source())
        ));
    }
    insta::assert_snapshot!(results.join("\n"), @r#"
    Scalar multiline: false -> false
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -outer:
    -  value: old
    +outer: {value: old,
    +  ⏎
    +  # detached
    +  }
     # trailing

    Scalar multiline: true -> false
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
    -outer:
    -  value: |-
    -    old
    +outer: {value: "old",
    +  ⏎
    +  # detached
    +  }
     # trailing
    "#);
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
    let candidate = editor.draft.clone().freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source_index = document.source.clone();
    let layout = RootLayout::new(&source_index, (&candidate).into(), &syntax);
    assert_eq!(
        plan(&source_index, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            splice::region(&document, document.source.root, true)
        )]
    );
    editor.finish().unwrap();
}
