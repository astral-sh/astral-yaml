use astral_yaml::Budget;
use astral_yaml::comments::CommentIndex;
use test_case::test_case;

#[test_case("[x, y]"; "sequence")]
#[test_case("{x: y}"; "mapping")]
fn slot_allocation_requires_budget_for_all_direct_children(collection: &str) {
    let source = format!("{collection} # note");
    let stream = astral_yaml::parse(&source).unwrap();
    // Reading the line and the root leaves room for only one child.
    let error = CommentIndex::with_work_budget(&stream, 0, &Budget::new(3)).unwrap_err();
    assert_eq!(error.span.slice(&source), Some(collection));
}

#[test]
fn looking_for_a_comment_owner_charges_each_descent() {
    let source = "[[[item # note\n]]]";
    let stream = astral_yaml::parse(source).unwrap();
    // After indexing both lines and all four nodes, locating the comment
    // visits the root and descends through three children to the scalar.
    let error = CommentIndex::with_work_budget(&stream, 0, &Budget::new(9)).unwrap_err();
    assert_eq!(error.span.slice(source), Some("# note"));
    assert!(CommentIndex::with_work_budget(&stream, 0, &Budget::new(10)).is_ok());
}

// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test]
fn an_omitted_value_does_not_use_its_parent_mapping_separator() {
    let source = "? ? leaf # inner entry\n: outer # outer value";
    let stream = astral_yaml::parse(source).unwrap();
    let index = CommentIndex::new(&stream, 0).unwrap();
    let owners: Vec<_> = index
        .direct()
        .map(|(id, comment)| {
            let token = stream.node(id).unwrap().span.slice(source).unwrap();
            format!("{:?} belongs to {token:?}", comment.comment.text.trim())
        })
        .collect();
    insta::assert_snapshot!(owners.join("\n"), @r#"
    "inner entry" belongs to ""
    "outer value" belongs to "outer"
    "#);
}
