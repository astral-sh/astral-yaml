#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-vuejs-compose-cf637ede"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "web" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "vuejs" [plain @4:16..4:21]
            "target" [plain @5:7..5:13]: "development" [plain @5:15..5:26]
          "ports" [plain @6:5..6:10]: sequence [block]
            - "8080:8080" [plain @7:9..7:18]
          "volumes" [plain @8:5..8:12]: sequence [block]
            - "./vuejs:/project" [plain @9:9..9:25]
            - "/project/node_modules" [plain @10:9..10:30]
    "#);
}
