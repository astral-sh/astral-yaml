#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-workflow-count-resourcequota-02adcf69"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "ResourceQuota" [plain @2:7..2:20]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "workflow-count" [plain @4:9..4:23]
        "annotations" [plain @5:3..5:14]: mapping [block]
          "workflows.argoproj.io/description" [plain @6:5..6:38]: literal [@6:40..22:1]
            "This resource quota prevents creation of more than a certain number of workflows being created in a namespace.\n"
            "\n"
            "The user will get an error when they try to create more. As the count includes both completed and incomplete\n"
            "workflows, and complete workflows do not make any significant demands of your cluster, you probably want to use it\n"
            "with a workflow GC strategy, so that a build up of completed workflows does not prevent you from creating and\n"
            "running new workflows.\n"
            "\n"
            "Argo Workflowws has feature called \"parallelism\" that limits the number of running workflows, ignoring completed\n"
            "workflows.\n"
            "\n"
            "<= v3.1 you can configure a global limit to the total number of running workflows in the cluster. This is suitable\n"
            "for single-tenancy set-up.\n"
            "\n"
            "For multi-tenancy set-up, where each tenant owns a single namespace, then in >= v3.1 you can globally configure a\n"
            "limit for the total number of running workflows within each namespace.\n"
      "spec" [plain @22:1..22:5]: mapping [block]
        "hard" [plain @23:3..23:7]: mapping [block]
          "count/workflows.argoproj.io" [plain @24:5..24:32]: "100" [double-quoted @24:34..24:39]
    "#);
}
