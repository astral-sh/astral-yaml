#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-mount-uv-lock-519c658c"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "volumes" [plain @24:5..24:12]: sequence [block]
            - mapping [block]
              "type" [plain @25:9..25:13]: "bind" [plain @25:15..25:19]
              "source" [plain @26:9..26:15]: "../../../uv.lock" [plain @26:17..26:33]
              "target" [plain @27:9..27:15]: "/opt/airflow/uv.lock" [plain @27:17..27:37]
    "#);
}
