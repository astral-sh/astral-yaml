#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-exporter-datadogexporter-examples-k8s-chart-roles-767c090b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "rbac.authorization.k8s.io/v1" [plain @1:13..1:41]
      "kind" [plain @2:1..2:5]: "ClusterRole" [plain @2:7..2:18]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "otel-collector-role" [plain @4:9..4:28]
      "rules" [plain @5:1..5:6]: sequence [block]
        - mapping [block]
          "apiGroups" [plain @6:3..6:12]: sequence [flow]
            - "" [double-quoted @6:15..6:17]
          "resources" [plain @7:3..7:12]: sequence [flow]
            - "pods" [double-quoted @7:15..7:21]
            - "namespaces" [double-quoted @7:23..7:35]
          "verbs" [plain @8:3..8:8]: sequence [flow]
            - "get" [double-quoted @8:11..8:16]
            - "watch" [double-quoted @8:18..8:25]
            - "list" [double-quoted @8:27..8:33]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @10:1..10:11]: "rbac.authorization.k8s.io/v1" [plain @10:13..10:41]
      "kind" [plain @11:1..11:5]: "ClusterRoleBinding" [plain @11:7..11:25]
      "metadata" [plain @12:1..12:9]: mapping [block]
        "name" [plain @13:3..13:7]: "otel-collector" [plain @13:9..13:23]
      "subjects" [plain @14:1..14:9]: sequence [block]
        - mapping [block]
          "kind" [plain @15:3..15:7]: "ServiceAccount" [plain @15:9..15:23]
          "name" [plain @16:3..16:7]: "otel-collector-account" [plain @16:9..16:31]
          "namespace" [plain @17:3..17:12]: "default" [plain @17:14..17:21]
      "roleRef" [plain @18:1..18:8]: mapping [block]
        "kind" [plain @19:3..19:7]: "ClusterRole" [plain @19:9..19:20]
        "name" [plain @20:3..20:7]: "otel-collector-role" [plain @20:9..20:28]
        "apiGroup" [plain @21:3..21:11]: "rbac.authorization.k8s.io" [plain @21:13..21:38]
    "#);
}
