#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googlecloudplatform-microservices-demo-kustomize-components-service-mesh-istio-frontend-18feca00"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @15:1..15:11]: "networking.istio.io/v1alpha3" [plain @15:13..15:41]
      "kind" [plain @16:1..16:5]: "VirtualService" [plain @16:7..16:21]
      "metadata" [plain @17:1..17:9]: mapping [block]
        "name" [plain @18:3..18:7]: "frontend" [plain @18:9..18:17]
      "spec" [plain @19:1..19:5]: mapping [block]
        "hosts" [plain @20:3..20:8]: sequence [block]
          - "frontend.default.svc.cluster.local" [double-quoted @21:5..21:41]
        "http" [plain @22:3..22:7]: sequence [block]
          - mapping [block]
            "route" [plain @23:5..23:10]: sequence [block]
              - mapping [block]
                "destination" [plain @24:7..24:18]: mapping [block]
                  "host" [plain @25:9..25:13]: "frontend" [plain @25:15..25:23]
                  "port" [plain @26:9..26:13]: mapping [block]
                    "number" [plain @27:11..27:17]: "80" [plain @27:19..27:21]
    "#);
}
