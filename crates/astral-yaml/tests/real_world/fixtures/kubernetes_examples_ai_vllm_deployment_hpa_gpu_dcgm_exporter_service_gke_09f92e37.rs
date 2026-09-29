#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-ai-vllm-deployment-hpa-gpu-dcgm-exporter-service-gke-09f92e37"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @12:1..12:11]: "v1" [plain @12:13..12:15]
      "kind" [plain @13:1..13:5]: "Service" [plain @13:7..13:14]
      "metadata" [plain @14:1..14:9]: mapping [block]
        "name" [plain @15:3..15:7]: "gke-managed-dcgm-exporter" [plain @15:9..15:34]
        "namespace" [plain @18:3..18:12]: "gke-managed-system" [plain @18:14..18:32]
        "labels" [plain @19:3..19:9]: mapping [block]
          "app.kubernetes.io/name" [plain @25:5..25:27]: "gke-managed-dcgm-exporter" [plain @25:29..25:54]
      "spec" [plain @26:1..26:5]: mapping [block]
        "selector" [plain @27:3..27:11]: mapping [block]
          "app.kubernetes.io/name" [plain @31:5..31:27]: "gke-managed-dcgm-exporter" [plain @31:29..31:54]
        "ports" [plain @32:3..32:8]: sequence [block]
          - mapping [block]
            "name" [plain @35:7..35:11]: "metrics" [plain @35:13..35:20]
            "port" [plain @36:7..36:11]: "9400" [plain @36:13..36:17]
            "targetPort" [plain @37:7..37:17]: "9400" [plain @37:19..37:23]
    "#);
}
