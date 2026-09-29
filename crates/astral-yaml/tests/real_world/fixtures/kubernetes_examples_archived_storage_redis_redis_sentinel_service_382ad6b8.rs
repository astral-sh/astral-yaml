#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-storage-redis-redis-sentinel-service-382ad6b8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "labels" [plain @4:3..4:9]: mapping [block]
          "name" [plain @5:5..5:9]: "sentinel" [plain @5:11..5:19]
          "role" [plain @6:5..6:9]: "service" [plain @6:11..6:18]
        "name" [plain @7:3..7:7]: "redis-sentinel" [plain @7:9..7:23]
      "spec" [plain @8:1..8:5]: mapping [block]
        "ports" [plain @9:3..9:8]: sequence [block]
          - mapping [block]
            "port" [plain @10:7..10:11]: "26379" [plain @10:13..10:18]
            "targetPort" [plain @11:7..11:17]: "26379" [plain @11:19..11:24]
        "selector" [plain @12:3..12:11]: mapping [block]
          "redis-sentinel" [plain @13:5..13:19]: "true" [double-quoted @13:21..13:27]
    "#);
}
