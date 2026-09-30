#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-postgres-docker-compose-1d8146fb"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "proxy" [plain @3:3..3:8]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "../shared/envoy/Dockerfile" [plain @6:19..6:45]
            "args" [plain @7:7..7:11]: mapping [block]
              "ENVOY_VARIANT" [plain @8:9..8:22]: "contrib-dev" [plain @8:24..8:35]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "${PORT_ADMIN:-8001}:8001" [double-quoted @10:7..10:33]
        "postgres" [plain @12:3..12:11]: mapping [block]
          "build" [plain @13:5..13:10]: mapping [block]
            "context" [plain @14:7..14:14]: "../shared/postgres" [plain @14:16..14:34]
          "environment" [plain @15:5..15:16]: mapping [block]
            "POSTGRES_HOST_AUTH_METHOD" [plain @21:7..21:32]: "trust" [plain @21:34..21:39]
    "#);
}
