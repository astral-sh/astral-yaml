#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-smoke-test-spring-boot-smoke-test-grpc-server-oauth-src-main-resources-application-66d0fcb7"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "logging.level.org.springframework.security" [plain @1:1..1:43]: "TRACE" [plain @1:45..1:50]
      "spring" [plain @2:1..2:7]: mapping [block]
        "security" [plain @3:3..3:11]: mapping [block]
          "oauth2" [plain @4:5..4:11]: mapping [block]
            "authorizationserver" [plain @5:7..5:26]: mapping [block]
              "client" [plain @6:9..6:15]: mapping [block]
                "oidc-client" [plain @7:11..7:22]: mapping [block]
                  "registration" [plain @8:13..8:25]: mapping [block]
                    "client-id" [plain @9:15..9:24]: "spring" [double-quoted @9:26..9:34]
                    "client-secret" [plain @10:15..10:28]: "{noop}secret" [double-quoted @10:30..10:44]
                    "client-authentication-methods" [plain @11:15..11:44]: sequence [block]
                      - "client_secret_basic" [double-quoted @12:19..12:40]
                    "authorization-grant-types" [plain @13:15..13:40]: sequence [block]
                      - "client_credentials" [double-quoted @14:19..14:39]
                      - "refresh_token" [double-quoted @15:19..15:34]
                    "scopes" [plain @16:15..16:21]: sequence [block]
                      - "openid" [double-quoted @17:19..17:27]
                      - "profile" [double-quoted @18:19..18:28]
            "resourceserver" [plain @19:7..19:21]: mapping [block]
              "jwt" [plain @20:9..20:12]: mapping [block]
                "jwk-set-uri" [plain @21:11..21:22]: "http://localhost:8080/oauth2/jwks" [plain @21:24..21:57]
    "#);
}
