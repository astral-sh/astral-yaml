#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-fault-injection-docker-compose-338da9b6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "envoy" [plain @2:3..2:8]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "." [plain @4:16..4:17]
            "dockerfile" [plain @5:7..5:17]: "../shared/envoy/Dockerfile" [plain @5:19..5:45]
            "target" [plain @6:7..6:13]: "envoy-fault-injection" [plain @6:15..6:36]
          "volumes" [plain @7:5..7:12]: sequence [block]
            - "./runtime:/srv/runtime" [plain @8:7..8:29]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "9211:9211" [plain @10:7..10:16]
        "backend" [plain @12:3..12:10]: mapping [block]
          "build" [plain @13:5..13:10]: mapping [block]
            "context" [plain @14:7..14:14]: "." [plain @14:16..14:17]
            "dockerfile" [plain @15:7..15:17]: "Dockerfile-backend" [plain @15:19..15:37]
          "ports" [plain @16:5..16:10]: sequence [block]
            - "${PORT_PROXY:-8080}:80" [plain @17:7..17:29]
    "#);
}
