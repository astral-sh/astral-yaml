#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-integration-qdrant-14a71fd2"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "qdrant" [plain @19:3..19:9]: mapping [block]
          "image" [plain @20:5..20:10]: "qdrant/qdrant:v1.10.1" [plain @20:12..20:33]
          "labels" [plain @21:5..21:11]: mapping [block]
            "org.apache.airflow.breeze" [plain @22:7..22:32]: "true" [double-quoted @22:34..22:40]
            "org.apache.airflow.breeze.worktree" [plain @23:7..23:41]: "${BREEZE_WORKTREE_PATH:-}" [double-quoted @23:43..23:70]
            "breeze.description" [plain @24:7..24:25]: "Integration required for Qdrant tests." [double-quoted @24:27..24:67]
          "ports" [plain @25:5..25:10]: sequence [block]
            - "6333:6333" [double-quoted @26:9..26:20]
            - "6334:6334" [double-quoted @27:9..27:20]
          "restart" [plain @28:5..28:12]: "on-failure" [double-quoted @28:14..28:26]
        "airflow" [plain @30:3..30:10]: mapping [block]
          "environment" [plain @31:5..31:16]: sequence [block]
            - "INTEGRATION_QDRANT=true" [plain @32:9..32:32]
          "depends_on" [plain @33:5..33:15]: mapping [block]
            "qdrant" [plain @34:7..34:13]: mapping [block]
              "condition" [plain @35:9..35:18]: "service_started" [plain @35:20..35:35]
    "#);
}
