#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-deployment-oss-dagster-pg-1975ca11"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "storage" [plain @1:1..1:8]: mapping [block]
        "postgres" [plain @2:3..2:11]: mapping [block]
          "postgres_db" [plain @3:5..3:16]: mapping [block]
            "username" [plain @4:7..4:15]: "my_username" [plain @4:17..4:28]
            "password" [plain @5:7..5:15]: "my_password" [plain @5:17..5:28]
            "hostname" [plain @6:7..6:15]: "my_hostname" [plain @6:17..6:28]
            "db_name" [plain @7:7..7:14]: "my_database" [plain @7:16..7:27]
            "port" [plain @8:7..8:11]: "5432" [plain @8:13..8:17]
    "#);
}
