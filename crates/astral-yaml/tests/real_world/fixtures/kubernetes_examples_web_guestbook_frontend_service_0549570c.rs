#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-web-guestbook-frontend-service-0549570c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "frontend" [plain @4:9..4:17]
        "labels" [plain @5:3..5:9]: mapping [block]
          "app" [plain @6:5..6:8]: "guestbook" [plain @6:10..6:19]
          "tier" [plain @7:5..7:9]: "frontend" [plain @7:11..7:19]
      "spec" [plain @8:1..8:5]: mapping [block]
        "type" [plain @10:3..10:7]: "NodePort" [plain @10:9..10:17]
        "ports" [plain @14:3..14:8]: sequence [block]
          - mapping [block]
            "port" [plain @15:5..15:9]: "80" [plain @15:11..15:13]
        "selector" [plain @16:3..16:11]: mapping [block]
          "app" [plain @17:5..17:8]: "guestbook" [plain @17:10..17:19]
          "tier" [plain @18:5..18:9]: "frontend" [plain @18:11..18:19]
    "#);
}
