#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-fastapi-compose-ad47926c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "api" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "." [plain @4:16..4:17]
            "target" [plain @5:7..5:13]: "builder" [plain @5:15..5:22]
          "container_name" [plain @6:5..6:19]: "fastapi-application" [plain @6:21..6:40]
          "environment" [plain @7:5..7:16]: mapping [block]
            "PORT" [plain @8:7..8:11]: "8000" [plain @8:13..8:17]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "8000:8000" [single-quoted @10:9..10:20]
          "restart" [plain @11:5..11:12]: "no" [double-quoted @11:14..11:18]
    "#);
}
