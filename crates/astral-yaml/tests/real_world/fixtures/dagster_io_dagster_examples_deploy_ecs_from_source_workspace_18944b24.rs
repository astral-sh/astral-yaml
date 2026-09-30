#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-deploy-ecs-from-source-workspace-18944b24"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "load_from" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "grpc_server" [plain @3:5..3:16]: mapping [block]
            "host" [plain @4:7..4:11]: "user_code" [plain @4:13..4:22]
            "port" [plain @5:7..5:11]: "4000" [plain @5:13..5:17]
            "location_name" [plain @6:7..6:20]: "example_jobs" [double-quoted @6:22..6:36]
    "#);
}
