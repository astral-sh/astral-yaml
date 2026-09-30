#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-smoke-test-spring-boot-smoke-test-reactive-oauth2-resource-server-src-main-resources-application-c0bd348c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "spring" [plain @1:1..1:7]: mapping [block]
        "security" [plain @2:3..2:11]: mapping [block]
          "oauth2" [plain @3:5..3:11]: mapping [block]
            "resourceserver" [plain @4:7..4:21]: mapping [block]
              "jwt" [plain @5:9..5:12]: mapping [block]
                "jwk-set-uri" [plain @7:11..7:22]: "https://example.com/oauth2/default/v1/keys" [plain @7:24..7:66]
    "#);
}
