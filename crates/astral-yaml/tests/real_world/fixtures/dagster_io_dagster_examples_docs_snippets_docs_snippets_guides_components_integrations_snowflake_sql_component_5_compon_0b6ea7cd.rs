#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-snowflake-sql-component-5-compon-0b6ea7cd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster.TemplatedSqlComponent" [plain @1:7..1:36]
      "attributes" [plain @3:1..3:11]: mapping [flow]
    "#);
}
