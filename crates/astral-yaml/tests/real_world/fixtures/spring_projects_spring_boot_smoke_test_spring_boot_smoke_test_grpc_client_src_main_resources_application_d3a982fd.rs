#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-smoke-test-spring-boot-smoke-test-grpc-client-src-main-resources-application-d3a982fd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "spring" [plain @1:1..1:7]: mapping [block]
        "grpc" [plain @2:3..2:7]: mapping [block]
          "client" [plain @3:5..3:11]: mapping [block]
            "channel" [plain @4:7..4:14]: mapping [block]
              "default" [plain @5:9..5:16]: mapping [block]
                "target" [plain @6:11..6:17]: "static://localhost:9090" [double-quoted @6:19..6:44]
    "#);
}
