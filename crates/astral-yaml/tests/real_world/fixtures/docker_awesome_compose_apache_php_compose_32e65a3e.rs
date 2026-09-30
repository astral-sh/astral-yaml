#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-apache-php-compose-32e65a3e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "web" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "app" [plain @4:16..4:19]
            "target" [plain @5:7..5:13]: "builder" [plain @5:15..5:22]
          "ports" [plain @6:5..6:10]: sequence [block]
            - "80:80" [single-quoted @7:9..7:16]
          "volumes" [plain @8:5..8:12]: sequence [block]
            - "./app:/var/www/html/" [plain @9:9..9:29]
    "#);
}
