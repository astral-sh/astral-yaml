#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-flask-redis-compose-e27a3588"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "redis" [plain @2:3..2:8]: mapping [block]
          "image" [plain @3:5..3:10]: "redislabs/redismod" [plain @3:12..3:30]
          "ports" [plain @4:5..4:10]: sequence [block]
            - "6379:6379" [single-quoted @5:9..5:20]
        "web" [plain @6:3..6:6]: mapping [block]
          "build" [plain @7:5..7:10]: mapping [block]
            "context" [plain @8:7..8:14]: "." [plain @8:16..8:17]
            "target" [plain @9:7..9:13]: "builder" [plain @9:15..9:22]
          "stop_signal" [plain @12:5..12:16]: "SIGINT" [plain @12:18..12:24]
          "ports" [plain @13:5..13:10]: sequence [block]
            - "8000:8000" [single-quoted @14:9..14:20]
          "volumes" [plain @15:5..15:12]: sequence [block]
            - ".:/code" [plain @16:9..16:16]
          "depends_on" [plain @17:5..17:15]: sequence [block]
            - "redis" [plain @18:9..18:14]
    "#);
}
