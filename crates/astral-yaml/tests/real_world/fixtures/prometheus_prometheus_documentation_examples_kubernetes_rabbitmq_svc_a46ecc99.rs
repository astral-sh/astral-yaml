#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-kubernetes-rabbitmq-svc-a46ecc99"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "rabbitmq" [plain @4:9..4:17]
        "labels" [plain @5:3..5:9]: mapping [block]
          "name" [plain @6:5..6:9]: "rabbitmq" [plain @6:11..6:19]
      "spec" [plain @7:1..7:5]: mapping [block]
        "ports" [plain @8:3..8:8]: sequence [block]
          - mapping [block]
            "port" [plain @9:7..9:11]: "9090" [plain @9:13..9:17]
            "name" [plain @10:7..10:11]: "exporter" [plain @10:13..10:21]
            "targetPort" [plain @11:7..11:17]: "exporter" [plain @11:19..11:27]
            "protocol" [plain @12:7..12:15]: "TCP" [plain @12:17..12:20]
        "selector" [plain @13:3..13:11]: mapping [block]
          "app" [plain @14:5..14:8]: "rabbitmq" [plain @14:10..14:18]
    "#);
}
