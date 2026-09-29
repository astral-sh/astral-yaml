#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-powerbi-component-5-component-1d6f36a5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_powerbi.PowerBIWorkspaceComponent" [plain @1:7..1:48]
      "attributes" [plain @3:1..3:11]: mapping [flow]
    "#);
}
