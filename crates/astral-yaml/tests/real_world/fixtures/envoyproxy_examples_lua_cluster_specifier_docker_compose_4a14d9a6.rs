#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-lua-cluster-specifier-docker-compose-4a14d9a6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "proxy" [plain @3:3..3:8]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "../shared/envoy/Dockerfile" [plain @6:19..6:45]
          "ports" [plain @7:5..7:10]: sequence [block]
            - "${PORT_PROXY:-10000}:10000" [double-quoted @8:7..8:35]
    "#);
}
