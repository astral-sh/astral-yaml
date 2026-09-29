#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-backend-custom-49cafc10"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "environment" [plain @20:5..20:16]: sequence [block]
            - "BACKEND=custom" [plain @21:9..21:23]
            - "AIRFLOW__DATABASE__SQL_ALCHEMY_CONN" [plain @22:9..22:44]
    "#);
}
