#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-cors-backend-docker-compose-3b66924b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "front-envoy" [plain @3:3..3:14]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "../../shared/envoy/Dockerfile" [plain @6:19..6:48]
          "depends_on" [plain @7:5..7:15]: mapping [block]
            "backend-service" [plain @8:7..8:22]: mapping [block]
              "condition" [plain @9:9..9:18]: "service_healthy" [plain @9:20..9:35]
          "ports" [plain @10:5..10:10]: sequence [block]
            - "${PORT_BACKEND:-8002}:10000" [double-quoted @11:7..11:36]
            - "${PORT_STATS:-8003}:8001" [double-quoted @12:7..12:33]
        "backend-service" [plain @14:3..14:18]: mapping [block]
          "build" [plain @15:5..15:10]: mapping [block]
            "context" [plain @16:7..16:14]: "../../shared/python" [plain @16:16..16:35]
            "target" [plain @17:7..17:13]: "aiohttp-service" [plain @17:15..17:30]
          "volumes" [plain @18:5..18:12]: sequence [block]
            - "./service.py:/code/service.py" [plain @19:7..19:36]
    "#);
}
