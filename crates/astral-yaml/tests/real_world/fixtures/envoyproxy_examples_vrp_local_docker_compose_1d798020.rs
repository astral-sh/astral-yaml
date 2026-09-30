#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-vrp-local-docker-compose-1d798020"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "vrp" [plain @3:3..3:6]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "Dockerfile-vrp" [plain @6:19..6:33]
          "environment" [plain @7:5..7:16]: mapping [block]
            "ENVOY_EDGE_EXTRA_ARGS" [plain @8:7..8:28]: "" [double-quoted @8:30..8:32]
            "ENVOY_ORIGIN_EXTRA_ARGS" [plain @9:7..9:30]: "" [double-quoted @9:32..9:34]
          "ports" [plain @10:5..10:10]: sequence [block]
            - "10000:10000" [double-quoted @11:7..11:20]
    "#);
}
