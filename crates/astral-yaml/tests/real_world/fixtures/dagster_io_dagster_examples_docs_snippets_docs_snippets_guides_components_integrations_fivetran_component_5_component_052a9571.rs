#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-fivetran-component-5-component-052a9571"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_fivetran.FivetranAccountComponent" [plain @1:7..1:48]
      "attributes" [plain @3:1..3:11]: mapping [block]
        "workspace" [plain @4:3..4:12]: mapping [block]
          "account_id" [plain @5:5..5:15]: "test_account" [plain @5:17..5:29]
          "api_key" [plain @6:5..6:12]: "{{ env.FIVETRAN_API_KEY }}" [single-quoted @6:14..6:42]
          "api_secret" [plain @7:5..7:15]: "{{ env.FIVETRAN_API_SECRET }}" [single-quoted @7:17..7:48]
    "#);
}
