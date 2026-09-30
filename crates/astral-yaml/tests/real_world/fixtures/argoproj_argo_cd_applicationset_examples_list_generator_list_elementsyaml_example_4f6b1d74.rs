#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-list-generator-list-elementsyaml-example-4f6b1d74"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "key" [plain @1:1..1:4]: mapping [block]
        "components" [plain @2:3..2:13]: sequence [block]
          - mapping [block]
            "name" [plain @3:7..3:11]: "component1" [plain @3:13..3:23]
            "chart" [plain @4:7..4:12]: "podinfo" [plain @4:14..4:21]
            "version" [plain @5:7..5:14]: "6.3.2" [double-quoted @5:16..5:23]
            "releaseName" [plain @6:7..6:18]: "component1" [plain @6:20..6:30]
            "repoUrl" [plain @7:7..7:14]: "https://stefanprodan.github.io/podinfo" [double-quoted @7:16..7:56]
            "namespace" [plain @8:7..8:16]: "component1" [plain @8:18..8:28]
          - mapping [block]
            "name" [plain @9:7..9:11]: "component2" [plain @9:13..9:23]
            "chart" [plain @10:7..10:12]: "podinfo" [plain @10:14..10:21]
            "version" [plain @11:7..11:14]: "6.3.3" [double-quoted @11:16..11:23]
            "releaseName" [plain @12:7..12:18]: "component2" [plain @12:20..12:30]
            "repoUrl" [plain @13:7..13:14]: "ghcr.io/stefanprodan/charts" [double-quoted @13:16..13:45]
            "namespace" [plain @14:7..14:16]: "component2" [plain @14:18..14:28]
    "#);
}
