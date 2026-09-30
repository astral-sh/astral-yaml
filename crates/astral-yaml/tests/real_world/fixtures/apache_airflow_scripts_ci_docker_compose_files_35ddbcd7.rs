#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-files-35ddbcd7"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "volumes" [plain @20:5..20:12]: sequence [block]
            - "../../../files:/files:cached" [plain @21:9..21:37]
            - "../../../dist:/dist:cached" [plain @22:9..22:35]
    "#);
}
