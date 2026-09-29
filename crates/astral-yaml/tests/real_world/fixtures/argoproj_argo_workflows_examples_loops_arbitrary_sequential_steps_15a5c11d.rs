#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-loops-arbitrary-sequential-steps-15a5c11d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @5:1..5:11]: "argoproj.io/v1alpha1" [plain @5:13..5:33]
      "kind" [plain @6:1..6:5]: "Workflow" [plain @6:7..6:15]
      "metadata" [plain @7:1..7:9]: mapping [block]
        "labels" [plain @8:3..8:9]: mapping [block]
          "workflows.argoproj.io/no-test" [plain @9:5..9:34]: "environment" [double-quoted @9:36..9:49]
        "generateName" [plain @10:3..10:15]: "loop-arbitrary-sequential-steps-" [plain @10:17..10:49]
      "spec" [plain @11:1..11:5]: mapping [block]
        "entrypoint" [plain @12:3..12:13]: "loop-arbitrary-sequential-steps-example" [plain @12:15..12:54]
        "arguments" [plain @13:3..13:12]: mapping [block]
          "parameters" [plain @14:5..14:15]: sequence [block]
            - mapping [block]
              "name" [plain @15:9..15:13]: "step_params" [plain @15:15..15:26]
              "value" [plain @16:9..16:14]: literal [@16:16..25:1]
                "[\n"
                "  { \"exit_code\": 0, \"message\": \"succeeds 1\" },\n"
                "  { \"exit_code\": 0, \"message\": \"succeeds 2\" },\n"
                "  { \"exit_code\": 0, \"message\": \"succeeds 3\" },\n"
                "  { \"exit_code\": 1, \"message\": \"will fail and stop here\" },\n"
                "  { \"exit_code\": 0, \"message\": \"will not run\" },\n"
                "  { \"exit_code\": 0, \"message\": \"will not run\" }\n"
                "]\n"
        "templates" [plain @25:3..25:12]: sequence [block]
          - mapping [block]
            "name" [plain @26:7..26:11]: "loop-arbitrary-sequential-steps-example" [plain @26:13..26:52]
            "inputs" [plain @27:7..27:13]: mapping [block]
              "parameters" [plain @28:9..28:19]: sequence [block]
                - mapping [block]
                  "name" [plain @29:13..29:17]: "step_params" [plain @29:19..29:30]
            "parallelism" [plain @30:7..30:18]: "1" [plain @30:20..30:21]
            "failFast" [plain @31:7..31:15]: "true" [plain @31:17..31:21]
            "steps" [plain @32:7..32:12]: sequence [block]
              - sequence [block]
                - mapping [block]
                  "name" [plain @33:13..33:17]: "unit-step" [plain @33:19..33:28]
                  "template" [plain @34:13..34:21]: "unit-step-template" [plain @34:23..34:41]
                  "arguments" [plain @35:13..35:22]: mapping [block]
                    "parameters" [plain @36:15..36:25]: sequence [block]
                      - mapping [block]
                        "name" [plain @37:19..37:23]: "exit_code" [plain @37:25..37:34]
                        "value" [plain @38:19..38:24]: "{{item.exit_code}}" [double-quoted @38:26..38:46]
                      - mapping [block]
                        "name" [plain @39:19..39:23]: "message" [plain @39:25..39:32]
                        "value" [plain @40:19..40:24]: "{{item.message}}" [double-quoted @40:26..40:44]
                  "withParam" [plain @41:13..41:22]: "{{inputs.parameters.step_params}}" [double-quoted @41:24..41:59]
          - mapping [block]
            "name" [plain @43:7..43:11]: "unit-step-template" [plain @43:13..43:31]
            "inputs" [plain @44:7..44:13]: mapping [block]
              "parameters" [plain @45:9..45:19]: sequence [block]
                - mapping [block]
                  "name" [plain @46:13..46:17]: "exit_code" [plain @46:19..46:28]
                - mapping [block]
                  "name" [plain @47:13..47:17]: "message" [plain @47:19..47:26]
            "container" [plain @48:7..48:16]: mapping [block]
              "image" [plain @49:9..49:14]: "alpine:3.23" [plain @49:16..49:27]
              "command" [plain @50:9..50:16]: sequence [flow]
                - "/bin/sh" [single-quoted @50:20..50:29]
                - "-c" [single-quoted @50:31..50:35]
              "args" [plain @51:9..51:13]: sequence [flow]
                - "echo {{inputs.parameters.message}}; exit {{inputs.parameters.exit_code}}" [double-quoted @51:17..51:91]
    "#);
}
