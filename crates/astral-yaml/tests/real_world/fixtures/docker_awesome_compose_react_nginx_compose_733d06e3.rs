#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-react-nginx-compose-733d06e3"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "frontend" [plain @2:3..2:11]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "." [plain @4:16..4:17]
          "container_name" [plain @5:5..5:19]: "frontend" [plain @5:21..5:29]
          "ports" [plain @6:5..6:10]: sequence [block]
            - "80:80" [double-quoted @7:9..7:16]
    "#);
}
