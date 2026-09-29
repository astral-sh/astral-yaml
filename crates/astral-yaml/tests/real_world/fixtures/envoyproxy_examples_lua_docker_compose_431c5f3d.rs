#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-lua-docker-compose-431c5f3d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "proxy" [plain @3:3..3:8]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "../shared/envoy/Dockerfile" [plain @6:19..6:45]
            "target" [plain @7:7..7:13]: "envoy-lua" [plain @7:15..7:24]
          "ports" [plain @8:5..8:10]: sequence [block]
            - "${PORT_PROXY:-10000}:10000" [double-quoted @9:7..9:35]
        "web_service" [plain @11:3..11:14]: mapping [block]
          "build" [plain @12:5..12:10]: mapping [block]
            "context" [plain @13:7..13:14]: "../shared/echo" [plain @13:16..13:30]
    "#);
}
