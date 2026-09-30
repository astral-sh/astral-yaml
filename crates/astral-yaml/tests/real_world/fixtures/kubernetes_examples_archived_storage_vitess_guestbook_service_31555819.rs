#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-storage-vitess-guestbook-service-31555819"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "Service" [plain @1:7..1:14]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "guestbook" [plain @4:9..4:18]
        "labels" [plain @5:3..5:9]: mapping [block]
          "component" [plain @6:5..6:14]: "guestbook" [plain @6:16..6:25]
          "app" [plain @7:5..7:8]: "vitess" [plain @7:10..7:16]
      "spec" [plain @8:1..8:5]: mapping [block]
        "ports" [plain @9:3..9:8]: sequence [block]
          - mapping [block]
            "port" [plain @10:7..10:11]: "80" [plain @10:13..10:15]
            "targetPort" [plain @11:7..11:17]: "http-server" [plain @11:19..11:30]
        "selector" [plain @12:3..12:11]: mapping [block]
          "component" [plain @13:5..13:14]: "guestbook" [plain @13:16..13:25]
          "app" [plain @14:5..14:8]: "vitess" [plain @14:10..14:16]
        "type" [plain @15:3..15:7]: "LoadBalancer" [plain @15:9..15:21]
    "#);
}
