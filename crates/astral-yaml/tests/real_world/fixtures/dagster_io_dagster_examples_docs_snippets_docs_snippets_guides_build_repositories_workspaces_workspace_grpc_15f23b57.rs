#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-build-repositories-workspaces-workspace-grpc-15f23b57"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "load_from" [plain @3:1..3:10]: sequence [block]
        - mapping [block]
          "grpc_server" [plain @4:5..4:16]: mapping [block]
            "host" [plain @5:7..5:11]: "localhost" [plain @5:13..5:22]
            "port" [plain @6:7..6:11]: "4266" [plain @6:13..6:17]
            "location_name" [plain @7:7..7:20]: "my_grpc_server" [double-quoted @7:22..7:38]
    "#);
}
