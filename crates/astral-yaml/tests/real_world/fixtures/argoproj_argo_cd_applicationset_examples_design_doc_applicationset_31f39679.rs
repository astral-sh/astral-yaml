#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-design-doc-applicationset-31f39679"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @5:1..5:11]: "argoproj.io/v1alpha1" [plain @5:13..5:33]
      "kind" [plain @6:1..6:5]: "ApplicationSet" [plain @6:7..6:21]
      "metadata" [plain @7:1..7:9]: mapping [block]
        "name" [plain @8:3..8:7]: "guestbook" [plain @8:9..8:18]
      "spec" [plain @9:1..9:5]: mapping [block]
        "goTemplate" [plain @10:3..10:13]: "true" [plain @10:15..10:19]
        "goTemplateOptions" [plain @11:3..11:20]: sequence [flow]
          - "missingkey=error" [double-quoted @11:23..11:41]
        "generators" [plain @12:3..12:13]: sequence [block]
          - mapping [block]
            "clusters" [plain @13:5..13:13]: mapping [flow]
        "template" [plain @14:3..14:11]: mapping [block]
          "metadata" [plain @15:5..15:13]: mapping [block]
            "name" [plain @16:7..16:11]: "{{.name}}-guestbook" [single-quoted @16:13..16:34]
          "spec" [plain @17:5..17:9]: mapping [block]
            "source" [plain @18:7..18:13]: mapping [block]
              "repoURL" [plain @19:9..19:16]: "https://github.com/infra-team/cluster-deployments.git" [plain @19:18..19:71]
              "targetRevision" [plain @20:9..20:23]: "HEAD" [plain @20:25..20:29]
              "chart" [plain @21:9..21:14]: "guestbook" [plain @21:16..21:25]
            "destination" [plain @22:7..22:18]: mapping [block]
              "server" [plain @23:9..23:15]: "{{.server}}" [single-quoted @23:17..23:30]
              "namespace" [plain @24:9..24:18]: "guestbook" [plain @24:20..24:29]
    "#);
}
