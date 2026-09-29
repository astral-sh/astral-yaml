#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-devcontainer-4b6e0af2"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "stdin_open" [plain @20:5..20:15]: "true" [plain @20:17..20:21]
          "tty" [plain @21:5..21:8]: "true" [plain @21:10..21:14]
          "image" [plain @22:5..22:10]: "ghcr.io/apache/airflow/main/ci/python3.10" [plain @22:12..22:53]
          "env_file" [plain @23:5..23:13]: "devcontainer.env" [plain @23:15..23:31]
          "ports" [plain @24:5..24:10]: sequence [block]
            - "22:22" [double-quoted @25:9..25:16]
            - "8080:8080" [double-quoted @26:9..26:20]
            - "5555:5555" [double-quoted @27:9..27:20]
            - "6379:6379" [double-quoted @28:9..28:20]
          "cap_add" [plain @29:5..29:12]: sequence [block]
            - "SYS_PTRACE" [plain @30:9..30:19]
          "volumes" [plain @31:5..31:12]: sequence [block]
            - "/var/run/docker.sock:/var/run/docker.sock" [plain @33:9..33:50]
            - "/dev/urandom:/dev/random" [plain @34:9..34:33]
            - "../../../.:/opt/airflow" [plain @37:9..37:32]
    "#);
}
