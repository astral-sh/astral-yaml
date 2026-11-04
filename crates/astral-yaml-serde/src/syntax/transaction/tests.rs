use test_case::test_case;

use crate::syntax::draft::{Candidate, same_style};
use crate::syntax::framing::RootLayout;
use crate::syntax::plan::{Changes, Plan};
use crate::syntax::source::Source;
use crate::syntax::splice::{self, OuterComments};
use crate::syntax::view::Syntax;
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, ParseOptions, ScalarStyle};
use astral_yaml_dev::{edit_diff, indoc};

use super::{DocumentSyntax, Editor, Layout, NodeKind, Span};

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
    let document = DocumentSyntax::parse(source).unwrap();
    let mut editor = Editor::new(&document);
    let value = editor.mapping_entries(editor.root()).unwrap()[0].1;
    editor.replace(value, "requested").unwrap();

    // A broken planning or rendering pass must not change the draft against
    // which commit is checked. Deliberately corrupt only the rendering copy.
    let candidate = editor.draft.freeze().unwrap();
    let mut planned = Syntax::from(&candidate).nodes.to_vec();
    planned[value.index()].kind = NodeKind::Scalar {
        value: "incorrect".to_owned(),
        style: ScalarStyle::Plain,
    };
    let planned = Syntax {
        nodes: &planned,
        root: document.source.root,
    };
    let source = &document.source;
    let output = plan(source, &candidate, planned)
        .render(&Layout::new(planned))
        .unwrap();
    let error = candidate.commit(&output).unwrap_err();
    insta::assert_snapshot!(
        format!("{}\nCommit: {}", edit_diff(&source.text, &output), error.message),
        @"
    @@ -1 +1 @@
    -value: old
    +value: incorrect

    Commit: emission changed YAML content or alias identity
    ");
}

#[test]
fn overlapping_rewrite_regions_expand_to_their_common_ancestor() {
    let mut document = DocumentSyntax::parse("[[a, b], c]").unwrap();
    let first = astral_yaml::NodeId::from_index(
        document
            .source
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "a"))
            .unwrap(),
    );
    let last = astral_yaml::NodeId::from_index(
        document
            .source
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "c"))
            .unwrap(),
    );
    // The patch planner must coalesce overlapping retained ranges, including
    // ranges expanded by trivia ownership, before slicing the original source.
    document.source.nodes[first.index()].span = Span::new(2, 10).unwrap();
    let mut editor = Editor::new(&document);
    editor.replace(first, "new_a").unwrap();
    editor.replace(last, "new_c").unwrap();
    let candidate = editor.draft.freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source = &document.source;
    let layout = RootLayout::new(source, (&candidate).into(), &syntax);
    assert_eq!(
        plan(source, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            document.source.nodes[document.source.root.index()].span
        )]
    );
}

#[test]
fn an_exhausted_work_budget_allows_an_empty_commit_but_not_a_rewrite() {
    let options = ParseOptions {
        max_nodes: 1,
        ..ParseOptions::default()
    };
    let mut document = DocumentSyntax::parse_with_options("one", options).unwrap();
    document
        .edit(|root| {
            root.node()?; // Spend the entire work budget on a read.
            Ok(())
        })
        .unwrap();

    let error = document
        .edit(|mut root| {
            // Changing the style spends the budget. Only commit should fail.
            root.set_scalar_style(ScalarStyle::SingleQuoted).unwrap();
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.message, "editing resource limit exceeded");
    assert_eq!(document.source(), "one");
}

#[test]
fn named_alias_key_lookup_charges_work_for_the_target() {
    let options = ParseOptions {
        max_nodes: 5,
        ..ParseOptions::default()
    };
    let document =
        DocumentSyntax::parse_with_options("{anchor: &a foo, ? *a : 2}", options).unwrap();
    let editor = Editor::new(&document);
    let root = editor.root();
    editor.node(root).unwrap();
    editor.node(root).unwrap();
    // The remaining budget covers the mapping and both keys, but not the alias target.
    let error = editor.string_field(root, "foo").unwrap_err();
    assert_eq!(error.message, "editing resource limit exceeded");
}

#[test]
fn inserting_outer_comments_obeys_the_output_limit() {
    let source = indoc! {r"
                # old
        value
    "};
    let options = ParseOptions {
        max_input_bytes: source.len(),
        ..ParseOptions::default()
    };
    let mut document = DocumentSyntax::parse_with_options(source, options).unwrap();
    let error = document
        .edit(|mut root| {
            for _ in 0..3 {
                root.append_comment(" new", CommentPlacement::Leading)?;
            }
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.message, "emitted YAML exceeds input byte limit");
    assert_eq!(document.source(), source);
}

#[test]
fn rewriting_only_an_outer_comment_obeys_the_work_limit() {
    let source = "# old\nvalue\n";
    let options = ParseOptions {
        max_nodes: 2,
        ..ParseOptions::default()
    };
    let mut document = DocumentSyntax::parse_with_options(source, options).unwrap();
    let error = document
        .edit(|mut root| {
            root.replace_comment(0, " new").unwrap(); // Spend the budget before commit.
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.message, "editing resource limit exceeded");
    assert_eq!(document.source(), source);
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
    // Scalar and collection styles distinguish presentations of the same content.
    // https://yaml.org/spec/1.2.2/#3231-node-styles
    let left = DocumentSyntax::parse(left).unwrap();
    let right = DocumentSyntax::parse(right).unwrap();
    assert_eq!(
        same_style(
            &left.source.nodes[left.source.root.index()].kind,
            &right.source.nodes[right.source.root.index()].kind
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
        let mut document = DocumentSyntax::parse(source).unwrap();
        document
            .edit_syntax(|editor| {
                let target = editor.mapping_entries(editor.root())?[0].1;
                editor.set_collection_style(target, CollectionStyle::Flow)?;
                match editor.node(target)?.kind() {
                    NodeKind::Sequence { .. } => editor.insert_item(target, 1, "new").map(|_| ()),
                    _ => editor.insert_entry(target, 1, "added", "new").map(|_| ()),
                }
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,4 +1,4 @@
     outer:
    -# detached
    +  # detached
     ⏎
    -  key: old
    +  {key: old, added: new}

    @@ -1,4 +1,4 @@
     outer:
       # detached
     ⏎
    -    key: old
    +  {key: old, added: new}

    @@ -1,3 +1,2 @@
    -outer:
     # leading
    -  {key: old}
    +outer: {key: old, added: new}

    @@ -1,3 +1,2 @@
    -outer:
     # leading
    -  [old]
    +outer: [old, new]

    @@ -1,2 +1 @@
    -outer:
    -  [old] # inline
    +outer: [old, new] # inline

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
    // Keep chomping makes trailing empty lines part of the scalar value.
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let document = DocumentSyntax::parse(indoc! {r"
        outer:
          value: old

        # detached
    "})
    .unwrap();
    let mut editor = Editor::new(&document);
    let inner = editor.mapping_entries(editor.root()).unwrap()[0].1;
    let value = editor.mapping_entries(inner).unwrap()[0].1;
    editor
        .replace(
            value,
            indoc! {r"
                |+
                  new

            "},
        )
        .unwrap();
    let candidate = editor.draft.freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source = &document.source;
    let layout = RootLayout::new(source, (&candidate).into(), &syntax);
    assert_eq!(
        plan(source, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            splice::region(&document, document.source.root, OuterComments::Include)
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
        let document = DocumentSyntax::parse(source).unwrap();
        let mut editor = Editor::new(&document);
        let inner = editor.mapping_entries(editor.root()).unwrap()[0].1;
        let value = editor.mapping_entries(inner).unwrap()[0].1;
        let multiline_before =
            Layout::new(&document.source).multiline(value, editor.node(value).unwrap().kind());
        editor
            .insert_comment(inner, 0, " detached", CommentPlacement::Detached)
            .unwrap();
        let candidate = editor.draft.clone().freeze().unwrap();
        let layout = Layout::new(&candidate);
        let multiline_after = layout.multiline(value, editor.node(value).unwrap().kind());
        let edited = editor.finish().unwrap();
        results.push(format!(
            "Scalar multiline: {multiline_before} -> {multiline_after}\n{}",
            edit_diff(source, edited.source())
        ));
    }
    insta::assert_snapshot!(results.join("\n"), @r#"
    Scalar multiline: false -> false
    @@ -1,3 +1,5 @@
    -outer:
    -  value: old
    +outer: {value: old,
    +  ⏎
    +  # detached
    +  }
     # trailing

    Scalar multiline: true -> false
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
    let document = DocumentSyntax::parse(indoc! {r"
        outer: {value: old}

           # root
    "})
    .unwrap();
    let mut editor = Editor::new(&document);
    let inner = editor.mapping_entries(editor.root()).unwrap()[0].1;
    let value = editor.mapping_entries(inner).unwrap()[0].1;
    editor.replace(value, "new").unwrap();
    let candidate = editor.draft.clone().freeze().unwrap();
    let syntax = Layout::new(&candidate);
    let source = &document.source;
    let layout = RootLayout::new(source, (&candidate).into(), &syntax);
    assert_eq!(
        plan(source, &candidate, (&candidate).into())
            .rewrite_regions(&layout, &syntax)
            .unwrap()
            .iter()
            .map(|rewrite| (rewrite.id, rewrite.boundary.body()))
            .collect::<Vec<_>>(),
        [(
            document.source.root,
            splice::region(&document, document.source.root, OuterComments::Include)
        )]
    );
    editor.finish().unwrap();
}
