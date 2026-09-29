#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-sparkjava-compose-b0f098e4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "sparkjava" [plain @2:3..2:12]: mapping [block]
          "build" [plain @3:5..3:10]: "sparkjava" [plain @3:12..3:21]
          "ports" [plain @4:5..4:10]: sequence [block]
            - "8080:8080" [plain @5:9..5:18]
    "#);
}
