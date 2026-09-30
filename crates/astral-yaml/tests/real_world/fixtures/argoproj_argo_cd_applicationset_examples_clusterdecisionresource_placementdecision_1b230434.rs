#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-clusterdecisionresource-placementdecision-1b230434"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @6:1..6:11]: "cluster.open-cluster-management.io/v1beta1" [plain @6:13..6:55]
      "kind" [plain @7:1..7:5]: "PlacementDecision" [plain @7:7..7:24]
      "metadata" [plain @8:1..8:9]: mapping [block]
        "name" [plain @9:3..9:7]: "test-placement" [plain @9:9..9:23]
        "labels" [plain @10:3..10:9]: mapping [block]
          "cluster.open-cluster-management.io/placement" [plain @11:5..11:49]: "test-placement" [plain @11:51..11:65]
      "status" [plain @12:1..12:7]: mapping [block]
        "decisions" [plain @13:3..13:12]: sequence [block]
          - mapping [block]
            "clusterName" [plain @14:5..14:16]: "cluster-01" [plain @14:18..14:28]
          - mapping [block]
            "clusterName" [plain @15:5..15:16]: "cluster-02" [plain @15:18..15:28]
    "#);
}
