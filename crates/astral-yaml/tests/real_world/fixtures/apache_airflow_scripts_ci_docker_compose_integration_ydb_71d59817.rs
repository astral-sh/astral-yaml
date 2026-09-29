#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-integration-ydb-71d59817"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "version" [plain @18:1..18:8]: "3.8" [double-quoted @18:10..18:15]
      "services" [plain @19:1..19:9]: mapping [block]
        "ydb" [plain @20:3..20:6]: mapping [block]
          "image" [plain @21:5..21:10]: "cr.yandex/yc/yandex-docker-local-ydb:latest" [plain @21:12..21:55]
          "labels" [plain @22:5..22:11]: mapping [block]
            "org.apache.airflow.breeze" [plain @23:7..23:32]: "true" [double-quoted @23:34..23:40]
            "org.apache.airflow.breeze.worktree" [plain @24:7..24:41]: "${BREEZE_WORKTREE_PATH:-}" [double-quoted @24:43..24:70]
            "breeze.description" [plain @25:7..25:25]: "Integration required for YDB tests." [double-quoted @25:27..25:64]
          "hostname" [plain @26:5..26:13]: "ydb" [plain @26:15..26:18]
          "environment" [plain @27:5..27:16]: mapping [block]
            "YDB_USE_IN_MEMORY_PDISKS" [plain @28:7..28:31]: "true" [plain @28:33..28:37]
            "GRPC_PORT" [plain @29:7..29:16]: "2136" [plain @29:18..29:22]
          "restart" [plain @30:5..30:12]: "on-failure" [double-quoted @30:14..30:26]
          "ports" [plain @31:5..31:10]: sequence [block]
            - "8765:8765" [double-quoted @32:9..32:20]
        "airflow" [plain @34:3..34:10]: mapping [block]
          "environment" [plain @35:5..35:16]: sequence [block]
            - "INTEGRATION_YDB=true" [plain @36:9..36:29]
          "depends_on" [plain @37:5..37:15]: mapping [block]
            "ydb" [plain @38:7..38:10]: mapping [block]
              "condition" [plain @39:9..39:18]: "service_started" [plain @39:20..39:35]
    "#);
}
