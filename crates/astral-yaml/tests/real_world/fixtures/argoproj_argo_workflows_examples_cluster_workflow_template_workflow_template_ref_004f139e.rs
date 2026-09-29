#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-cluster-workflow-template-workflow-template-ref-004f139e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "generateName" [plain @4:3..4:15]: "cluster-workflow-template-hello-world-" [plain @4:17..4:55]
      "spec" [plain @5:1..5:5]: mapping [block]
        "workflowTemplateRef" [plain @6:3..6:22]: mapping [block]
          "name" [plain @7:5..7:9]: "cluster-workflow-template-submittable" [plain @7:11..7:48]
          "clusterScope" [plain @8:5..8:17]: "true" [plain @8:19..8:23]
    "#);
}
