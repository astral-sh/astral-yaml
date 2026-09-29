#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-airbyte-component-5-component-178c563e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_airbyte.AirbyteWorkspaceComponent" [plain @1:7..1:48]
      "attributes" [plain @3:1..3:11]: mapping [block]
        "workspace" [plain @4:3..4:12]: mapping [block]
          "workspace_id" [plain @5:5..5:17]: "test_workspace" [plain @5:19..5:33]
          "client_id" [plain @6:5..6:14]: "{{ env.AIRBYTE_CLIENT_ID }}" [single-quoted @6:16..6:45]
          "client_secret" [plain @7:5..7:18]: "{{ env.AIRBYTE_CLIENT_SECRET }}" [single-quoted @7:20..7:53]
    "#);
}
