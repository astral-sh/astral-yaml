#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-storage-vitess-vtgate-service-21e36d57"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "Service" [plain @1:7..1:14]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "vtgate" [plain @4:9..4:15]
        "labels" [plain @5:3..5:9]: mapping [block]
          "component" [plain @6:5..6:14]: "vtgate" [plain @6:16..6:22]
          "app" [plain @7:5..7:8]: "vitess" [plain @7:10..7:16]
      "spec" [plain @8:1..8:5]: mapping [block]
        "ports" [plain @9:3..9:8]: sequence [block]
          - mapping [block]
            "port" [plain @10:7..10:11]: "15001" [plain @10:13..10:18]
        "selector" [plain @11:3..11:11]: mapping [block]
          "component" [plain @12:5..12:14]: "vtgate" [plain @12:16..12:22]
          "app" [plain @13:5..13:8]: "vitess" [plain @13:10..13:16]
        "type" [plain @14:3..14:7]: "LoadBalancer" [plain @14:9..14:21]
    "#);
}
