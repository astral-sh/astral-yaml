#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-forward-credentials-11c204da"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @18:1..18:9]: mapping [block]
        "airflow" [plain @19:3..19:10]: mapping [block]
          "environment" [plain @26:5..26:16]: sequence [block]
            - "GITHUB_TOKEN" [plain @27:9..27:21]
          "volumes" [plain @28:5..28:12]: sequence [block]
            - "${HOME}/.aws:/root/.aws:cached" [plain @29:9..29:39]
            - "${HOME}/.azure:/root/.azure:cached" [plain @30:9..30:43]
            - "${HOME}/.config:/root/.config:cached" [plain @31:9..31:45]
            - "${HOME}/.docker:/root/.docker:cached" [plain @32:9..32:45]
            - "${HOME}/.snowsql:/root/.snowsql:cached" [plain @33:9..33:47]
            - "${HOME}/.ssh:/root/.ssh:cached" [plain @34:9..34:39]
    "#);
}
