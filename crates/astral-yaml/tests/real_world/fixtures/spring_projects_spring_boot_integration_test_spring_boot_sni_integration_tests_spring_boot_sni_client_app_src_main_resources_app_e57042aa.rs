#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-integration-test-spring-boot-sni-integration-tests-spring-boot-sni-client-app-src-main-resources-app-e57042aa"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "spring" [plain @1:1..1:7]: mapping [block]
        "ssl" [plain @2:3..2:6]: mapping [block]
          "bundle" [plain @3:5..3:11]: mapping [block]
            "pem" [plain @4:7..4:10]: mapping [block]
              "server" [plain @5:9..5:15]: mapping [block]
                "truststore" [plain @6:11..6:21]: mapping [block]
                  "certificate" [plain @7:13..7:24]: "classpath:ca/test-ca.crt" [double-quoted @7:26..7:52]
    "#);
}
