#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-cluster-cluster-example-fasttemplate-3d2c66cc"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "ApplicationSet" [plain @2:7..2:21]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "guestbook" [plain @4:9..4:18]
      "spec" [plain @5:1..5:5]: mapping [block]
        "generators" [plain @6:3..6:13]: sequence [block]
          - mapping [block]
            "clusters" [plain @7:5..7:13]: mapping [flow]
        "template" [plain @8:3..8:11]: mapping [block]
          "metadata" [plain @9:5..9:13]: mapping [block]
            "name" [plain @10:7..10:11]: "{{name}}-guestbook" [single-quoted @10:13..10:33]
          "spec" [plain @11:5..11:9]: mapping [block]
            "project" [plain @12:7..12:14]: "default" [double-quoted @12:16..12:25]
            "source" [plain @13:7..13:13]: mapping [block]
              "repoURL" [plain @14:9..14:16]: "https://github.com/argoproj/argocd-example-apps/" [plain @14:18..14:66]
              "targetRevision" [plain @15:9..15:23]: "HEAD" [plain @15:25..15:29]
              "path" [plain @16:9..16:13]: "guestbook" [plain @16:15..16:24]
            "destination" [plain @17:7..17:18]: mapping [block]
              "server" [plain @18:9..18:15]: "{{server}}" [single-quoted @18:17..18:29]
              "namespace" [plain @19:9..19:18]: "guestbook" [plain @19:20..19:29]
    "#);
}
