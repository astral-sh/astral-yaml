#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-web-guestbook-redis-replica-service-1f73d0c0"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "redis-replica" [plain @4:9..4:22]
        "labels" [plain @5:3..5:9]: mapping [block]
          "app" [plain @6:5..6:8]: "redis" [plain @6:10..6:15]
          "role" [plain @7:5..7:9]: "replica" [plain @7:11..7:18]
          "tier" [plain @8:5..8:9]: "backend" [plain @8:11..8:18]
      "spec" [plain @9:1..9:5]: mapping [block]
        "ports" [plain @10:3..10:8]: sequence [block]
          - mapping [block]
            "port" [plain @11:5..11:9]: "6379" [plain @11:11..11:15]
        "selector" [plain @12:3..12:11]: mapping [block]
          "app" [plain @13:5..13:8]: "redis" [plain @13:10..13:15]
          "role" [plain @14:5..14:9]: "replica" [plain @14:11..14:18]
          "tier" [plain @15:5..15:9]: "backend" [plain @15:11..15:18]
    "#);
}
