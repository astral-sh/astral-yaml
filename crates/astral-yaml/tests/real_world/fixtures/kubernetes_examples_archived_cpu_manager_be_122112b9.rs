#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-cpu-manager-be-122112b9"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Pod" [plain @2:7..2:10]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "be" [plain @4:9..4:11]
      "spec" [plain @5:1..5:5]: mapping [block]
        "containers" [plain @6:3..6:13]: sequence [block]
          - mapping [block]
            "image" [plain @7:5..7:10]: "quay.io/connordoyle/cpuset-visualizer" [plain @7:12..7:49]
            "name" [plain @8:5..8:9]: "be" [plain @8:11..8:13]
    "#);
}
