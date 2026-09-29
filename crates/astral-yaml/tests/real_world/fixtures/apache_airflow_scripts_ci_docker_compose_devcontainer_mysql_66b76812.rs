#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-devcontainer-mysql-66b76812"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "mysql" [plain @19:3..19:8]: mapping [block]
          "image" [plain @20:5..20:10]: "mysql:8.0" [plain @20:12..20:21]
          "ports" [plain @21:5..21:10]: sequence [block]
            - "3306:3306" [double-quoted @22:9..22:20]
      "volumes" [plain @23:1..23:8]: mapping [block]
        "mysql-db-volume" [plain @24:3..24:18]: "" [plain @24:19..24:19]
    "#);
}
