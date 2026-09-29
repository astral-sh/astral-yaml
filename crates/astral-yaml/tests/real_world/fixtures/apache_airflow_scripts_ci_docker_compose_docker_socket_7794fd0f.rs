#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-docker-socket-7794fd0f"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "volumes" [plain @20:5..20:12]: sequence [block]
            - "/var/run/docker.sock:/var/run/docker.sock" [plain @25:9..25:50]
    "#);
}
