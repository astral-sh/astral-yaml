#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-steps-inline-workflow-24ea0eaa"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "generateName" [plain @4:3..4:15]: "steps-inline-" [plain @4:17..4:30]
        "annotations" [plain @5:3..5:14]: mapping [block]
          "workflows.argoproj.io/description" [plain @6:5..6:38]: literal [@6:40..8:1]
            "This workflow demonstrates running a steps with inline templates.\n"
          "workflows.argoproj.io/version" [plain @8:5..8:34]: ">= 3.2.0" [double-quoted @8:36..8:46]
      "spec" [plain @9:1..9:5]: mapping [block]
        "entrypoint" [plain @10:3..10:13]: "main" [plain @10:15..10:19]
        "templates" [plain @11:3..11:12]: sequence [block]
          - mapping [block]
            "name" [plain @12:7..12:11]: "main" [plain @12:13..12:17]
            "steps" [plain @13:7..13:12]: sequence [block]
              - sequence [block]
                - mapping [block]
                  "name" [plain @14:13..14:17]: "a" [plain @14:19..14:20]
                  "inline" [plain @15:13..15:19]: mapping [block]
                    "container" [plain @16:15..16:24]: mapping [block]
                      "image" [plain @17:17..17:22]: "argoproj/argosay:v2" [plain @17:24..17:43]
    "#);
}
