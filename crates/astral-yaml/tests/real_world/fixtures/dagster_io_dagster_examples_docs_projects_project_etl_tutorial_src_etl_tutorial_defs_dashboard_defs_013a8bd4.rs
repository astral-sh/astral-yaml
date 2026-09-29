#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-projects-project-etl-tutorial-src-etl-tutorial-defs-dashboard-defs-013a8bd4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_evidence.EvidenceProject" [plain @1:7..1:39]
      "attributes" [plain @3:1..3:11]: mapping [block]
        "project_path" [plain @4:3..4:15]: "../../../../dashboard" [plain @4:17..4:38]
        "asset" [plain @5:3..5:8]: mapping [block]
          "key" [plain @6:5..6:8]: "dashboard" [plain @6:10..6:19]
          "deps" [plain @7:5..7:9]: sequence [block]
            - "target/main/orders" [plain @8:9..8:27]
            - "target/main/customers" [plain @9:9..9:30]
        "deploy_command" [plain @10:3..10:17]: "echo \"Dashboard built at $EVIDENCE_BUILD_PATH\"" [single-quoted @10:19..10:67]
    "#);
}
