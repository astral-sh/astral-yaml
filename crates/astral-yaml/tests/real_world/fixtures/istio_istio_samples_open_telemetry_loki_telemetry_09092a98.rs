#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-open-telemetry-loki-telemetry-09092a98"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "telemetry.istio.io/v1" [plain @1:13..1:34]
      "kind" [plain @2:1..2:5]: "Telemetry" [plain @2:7..2:16]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "mesh-logging" [plain @4:9..4:21]
        "namespace" [plain @5:3..5:12]: "istio-system" [plain @5:14..5:26]
      "spec" [plain @6:1..6:5]: mapping [block]
        "accessLogging" [plain @7:3..7:16]: sequence [block]
          - mapping [block]
            "providers" [plain @8:7..8:16]: sequence [block]
              - mapping [block]
                "name" [plain @9:11..9:15]: "otel" [plain @9:17..9:21]
    "#);
}
