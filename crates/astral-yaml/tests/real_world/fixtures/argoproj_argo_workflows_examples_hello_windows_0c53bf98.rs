#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-hello-windows-0c53bf98"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "labels" [plain @4:3..4:9]: mapping [block]
          "workflows.argoproj.io/no-test" [plain @5:5..5:34]: "environment" [double-quoted @5:36..5:49]
        "generateName" [plain @6:3..6:15]: "hello-windows-" [plain @6:17..6:31]
      "spec" [plain @7:1..7:5]: mapping [block]
        "entrypoint" [plain @8:3..8:13]: "hello-win" [plain @8:15..8:24]
        "templates" [plain @9:3..9:12]: sequence [block]
          - mapping [block]
            "name" [plain @10:7..10:11]: "hello-win" [plain @10:13..10:22]
            "nodeSelector" [plain @11:7..11:19]: mapping [block]
              "kubernetes.io/os" [plain @12:9..12:25]: "windows" [plain @12:27..12:34]
            "container" [plain @13:7..13:16]: mapping [block]
              "image" [plain @14:9..14:14]: "mcr.microsoft.com/windows/nanoserver:1809" [plain @14:16..14:57]
              "command" [plain @15:9..15:16]: sequence [flow]
                - "cmd" [double-quoted @15:19..15:24]
                - "/c" [double-quoted @15:26..15:30]
              "args" [plain @16:9..16:13]: sequence [flow]
                - "echo" [double-quoted @16:16..16:22]
                - "Hello from Windows Container!" [double-quoted @16:24..16:55]
    "#);
}
