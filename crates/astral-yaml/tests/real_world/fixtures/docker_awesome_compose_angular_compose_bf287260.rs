#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-angular-compose-bf287260"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "web" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "angular" [plain @4:16..4:23]
            "target" [plain @5:7..5:13]: "builder" [plain @5:15..5:22]
          "ports" [plain @6:5..6:10]: sequence [block]
            - "4200:4200" [plain @7:9..7:18]
          "volumes" [plain @8:5..8:12]: sequence [block]
            - "./angular:/project" [plain @9:9..9:27]
            - "/project/node_modules" [plain @10:9..10:30]
    "#);
}
