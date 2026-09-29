#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-template-override-default-guestbook-ui-svc-3f99ec02"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "guestbook-ui" [plain @4:9..4:21]
      "spec" [plain @5:1..5:5]: mapping [block]
        "ports" [plain @6:3..6:8]: sequence [block]
          - mapping [block]
            "port" [plain @7:5..7:9]: "80" [plain @7:11..7:13]
            "targetPort" [plain @8:5..8:15]: "80" [plain @8:17..8:19]
        "selector" [plain @9:3..9:11]: mapping [block]
          "app" [plain @10:5..10:8]: "guestbook-ui" [plain @10:10..10:22]
    "#);
}
