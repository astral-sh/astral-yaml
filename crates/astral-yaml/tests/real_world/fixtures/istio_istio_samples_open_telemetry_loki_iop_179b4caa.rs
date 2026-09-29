#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-open-telemetry-loki-iop-179b4caa"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "install.istio.io/v1alpha1" [plain @1:13..1:38]
      "kind" [plain @2:1..2:5]: "IstioOperator" [plain @2:7..2:20]
      "spec" [plain @3:1..3:5]: mapping [block]
        "meshConfig" [plain @4:3..4:13]: mapping [block]
          "extensionProviders" [plain @5:5..5:23]: sequence [block]
            - mapping [block]
              "name" [plain @6:9..6:13]: "otel" [plain @6:15..6:19]
              "envoyOtelAls" [plain @7:9..7:21]: mapping [block]
                "service" [plain @8:11..8:18]: "opentelemetry-collector.istio-system.svc.cluster.local" [plain @8:20..8:74]
                "port" [plain @9:11..9:15]: "4317" [plain @9:17..9:21]
                "logFormat" [plain @10:11..10:20]: mapping [block]
                  "labels" [plain @11:13..11:19]: mapping [block]
                    "pod" [plain @12:15..12:18]: "%ENVIRONMENT(POD_NAME)%" [double-quoted @12:20..12:45]
                    "namespace" [plain @13:15..13:24]: "%ENVIRONMENT(POD_NAMESPACE)%" [double-quoted @13:26..13:56]
                    "cluster" [plain @14:15..14:22]: "%ENVIRONMENT(ISTIO_META_CLUSTER_ID)%" [double-quoted @14:24..14:62]
                    "mesh" [plain @15:15..15:19]: "%ENVIRONMENT(ISTIO_META_MESH_ID)%" [double-quoted @15:21..15:56]
    "#);
}
