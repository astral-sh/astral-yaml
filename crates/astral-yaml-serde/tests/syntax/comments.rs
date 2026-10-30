use crate::syntax::{DocumentSyntax, NodeKind};
use astral_yaml::CollectionStyle;
use astral_yaml_dev::indoc;

#[test]
fn explicit_collection_headers_bound_child_comment_ownership() {
    let mut attachments = Vec::new();
    for (name, source) in [
        (
            "flow sequence key",
            indoc! {r"
                {?
                  # collection
                  [item]: value}
            "},
        ),
        (
            "flow mapping key",
            indoc! {r"
                {?
                  # collection
                  {key: value}: value}
            "},
        ),
        (
            "anchored sequence key",
            indoc! {r"
                ?
                  # collection
                  &key
                  - item
                : value
            "},
        ),
        (
            "tagged mapping key",
            indoc! {r"
                ?
                  # collection
                  !!map
                  key: value
                : value
            "},
        ),
        (
            "flow sequence root",
            indoc! {r"
                # collection
                [item]
            "},
        ),
        (
            "flow mapping root",
            indoc! {r"
                # collection
                {key: value}
            "},
        ),
        (
            "implicit block sequence",
            indoc! {r"
                # item
                - item
            "},
        ),
        (
            "implicit block mapping",
            indoc! {r"
                # item
                key: value
            "},
        ),
    ] {
        attachments.extend(
            owned_comments(source)
                .into_iter()
                .map(|comment| format!("{name}: {comment}")),
        );
    }
    insta::assert_snapshot!(attachments.join("\n"), @r#"
    flow sequence key: flow sequence, Leading, " collection"
    flow mapping key: flow mapping, Leading, " collection"
    anchored sequence key: block sequence, Leading, " collection"
    tagged mapping key: block mapping, Leading, " collection"
    flow sequence root: flow sequence, Leading, " collection"
    flow mapping root: flow mapping, Leading, " collection"
    implicit block sequence: scalar, Leading, " item"
    implicit block mapping: scalar, Leading, " item"
    "#);
}

#[test]
fn outdented_prefixes_belong_to_the_collection() {
    let mut attachments = Vec::new();
    for (key, source) in [
        (
            true,
            indoc! {r"
                ?
                  # collection
                    - item
                : value
            "},
        ),
        (
            false,
            indoc! {r"
                key:
                  # collection
                    child: value
            "},
        ),
    ] {
        let document = DocumentSyntax::parse(source).unwrap();
        let NodeKind::Mapping { entries, .. } = document.node(document.root()).unwrap().kind()
        else {
            panic!("mapping")
        };
        let id = if key { entries[0].0 } else { entries[0].1 };
        attachments.push(
            document
                .node(id)
                .unwrap()
                .comments()
                .iter()
                .map(|comment| (comment.text().to_owned(), comment.placement()))
                .collect::<Vec<_>>(),
        );
    }
    insta::assert_debug_snapshot!(attachments, @r#"
    [
        [
            (
                " collection",
                Leading,
            ),
        ],
        [
            (
                " collection",
                Leading,
            ),
        ],
    ]
    "#);
}

fn owned_comments(source: &str) -> Vec<String> {
    let mut attachments = Vec::new();
    let document = DocumentSyntax::parse(source).unwrap();
    let mut pending = vec![document.root()];
    while let Some(id) = pending.pop() {
        let node = document.node(id).unwrap();
        let kind = match node.kind() {
            NodeKind::Sequence { items, style } => {
                pending.extend(items);
                if *style == CollectionStyle::Flow {
                    "flow sequence"
                } else {
                    "block sequence"
                }
            }
            NodeKind::Mapping { entries, style } => {
                for &(key, value) in entries {
                    pending.extend([key, value]);
                }
                if *style == CollectionStyle::Flow {
                    "flow mapping"
                } else {
                    "block mapping"
                }
            }
            NodeKind::Scalar { .. } => "scalar",
            NodeKind::Alias { .. } => "alias",
        };
        for comment in node.comments() {
            attachments.push(format!(
                "{kind}, {:?}, {:?}",
                comment.placement(),
                comment.text()
            ));
        }
    }

    attachments
}
