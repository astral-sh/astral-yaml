#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-template-override-engineering-dev-override-guestbook-ui-deployment-157d8e26"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "apps/v1" [plain @1:13..1:20]
      "kind" [plain @2:1..2:5]: "Deployment" [plain @2:7..2:17]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "guestbook-ui" [plain @4:9..4:21]
      "spec" [plain @5:1..5:5]: mapping [block]
        "replicas" [plain @6:3..6:11]: "1" [plain @6:13..6:14]
        "revisionHistoryLimit" [plain @7:3..7:23]: "3" [plain @7:25..7:26]
        "selector" [plain @8:3..8:11]: mapping [block]
          "matchLabels" [plain @9:5..9:16]: mapping [block]
            "app" [plain @10:7..10:10]: "guestbook-ui" [plain @10:12..10:24]
        "template" [plain @11:3..11:11]: mapping [block]
          "metadata" [plain @12:5..12:13]: mapping [block]
            "labels" [plain @13:7..13:13]: mapping [block]
              "app" [plain @14:9..14:12]: "guestbook-ui" [plain @14:14..14:26]
          "spec" [plain @15:5..15:9]: mapping [block]
            "containers" [plain @16:7..16:17]: sequence [block]
              - mapping [block]
                "image" [plain @17:9..17:14]: "quay.io/argoprojlabs/argocd-e2e-container:0.2" [plain @17:16..17:61]
                "name" [plain @18:9..18:13]: "guestbook-ui" [plain @18:15..18:27]
                "ports" [plain @19:9..19:14]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @20:11..20:24]: "80" [plain @20:26..20:28]
    "#);
}
