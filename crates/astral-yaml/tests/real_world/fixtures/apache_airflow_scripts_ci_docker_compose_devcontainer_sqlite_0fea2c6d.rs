#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-devcontainer-sqlite-0fea2c6d"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "environment" [plain @20:5..20:16]: sequence [block]
            - "BACKEND=sqlite" [plain @21:9..21:23]
            - "AIRFLOW__DATABASE__SQL_ALCHEMY_CONN=sqlite:////root/airflow/sqlite/airflow.db" [plain @22:9..22:86]
          "volumes" [plain @23:5..23:12]: sequence [block]
            - "sqlite-db-volume:/root/airflow/sqlite" [plain @24:9..24:46]
      "volumes" [plain @25:1..25:8]: mapping [block]
        "sqlite-db-volume" [plain @26:3..26:19]: "" [plain @26:20..26:20]
    "#);
}
