#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-build-partitions-backfills-workspace-0d99e227"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "load_from" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "python_module" [plain @2:5..2:18]: "docs_snippets.guides.build.partitions_backfills.repo" [plain @2:20..2:72]
    "#);
}
