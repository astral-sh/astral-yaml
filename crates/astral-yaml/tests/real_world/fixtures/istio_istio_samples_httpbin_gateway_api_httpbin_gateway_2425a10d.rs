#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-httpbin-gateway-api-httpbin-gateway-2425a10d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "gateway.networking.k8s.io/v1" [plain @1:13..1:41]
      "kind" [plain @2:1..2:5]: "Gateway" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "httpbin-gateway" [plain @4:9..4:24]
      "spec" [plain @5:1..5:5]: mapping [block]
        "gatewayClassName" [plain @6:3..6:19]: "istio" [plain @6:21..6:26]
        "listeners" [plain @7:3..7:12]: sequence [block]
          - mapping [block]
            "name" [plain @8:5..8:9]: "http" [plain @8:11..8:15]
            "port" [plain @9:5..9:9]: "80" [plain @9:11..9:13]
            "protocol" [plain @10:5..10:13]: "HTTP" [plain @10:15..10:19]
            "allowedRoutes" [plain @11:5..11:18]: mapping [block]
              "namespaces" [plain @12:7..12:17]: mapping [block]
                "from" [plain @13:9..13:13]: "All" [plain @13:15..13:18]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @15:1..15:11]: "gateway.networking.k8s.io/v1" [plain @15:13..15:41]
      "kind" [plain @16:1..16:5]: "HTTPRoute" [plain @16:7..16:16]
      "metadata" [plain @17:1..17:9]: mapping [block]
        "name" [plain @18:3..18:7]: "httpbin" [plain @18:9..18:16]
      "spec" [plain @19:1..19:5]: mapping [block]
        "parentRefs" [plain @20:3..20:13]: sequence [block]
          - mapping [block]
            "name" [plain @21:5..21:9]: "httpbin-gateway" [plain @21:11..21:26]
        "rules" [plain @22:3..22:8]: sequence [block]
          - mapping [block]
            "backendRefs" [plain @23:5..23:16]: sequence [block]
              - mapping [block]
                "name" [plain @24:7..24:11]: "httpbin" [plain @24:13..24:20]
                "port" [plain @25:7..25:11]: "8000" [plain @25:13..25:17]
    "#);
}
