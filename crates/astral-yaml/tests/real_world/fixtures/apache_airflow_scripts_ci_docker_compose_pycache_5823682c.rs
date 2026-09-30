#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-pycache-5823682c"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "volumes" [plain @20:5..20:12]: sequence [block]
            - mapping [block]
              "type" [plain @21:9..21:13]: "volume" [plain @21:15..21:21]
              "source" [plain @22:9..22:15]: "airflow-pycache-volume" [plain @22:17..22:39]
              "target" [plain @23:9..23:15]: "${PYTHONPYCACHEPREFIX}" [plain @23:17..23:39]
      "volumes" [plain @24:1..24:8]: mapping [block]
        "airflow-pycache-volume" [plain @25:3..25:25]: mapping [block]
          "external" [plain @26:5..26:13]: "true" [plain @26:15..26:19]
    "#);
}
