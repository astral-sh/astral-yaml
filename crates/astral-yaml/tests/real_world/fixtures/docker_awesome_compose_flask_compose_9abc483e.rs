#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-flask-compose-9abc483e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "web" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "app" [plain @4:16..4:19]
            "target" [plain @5:7..5:13]: "builder" [plain @5:15..5:22]
          "stop_signal" [plain @8:5..8:16]: "SIGINT" [plain @8:18..8:24]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "8000:8000" [single-quoted @10:9..10:20]
    "#);
}
