#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-cron-backfill-361e6fcc"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @3:1..3:11]: "argoproj.io/v1alpha1" [plain @3:13..3:33]
      "kind" [plain @4:1..4:5]: "WorkflowTemplate" [plain @4:7..4:23]
      "metadata" [plain @5:1..5:9]: mapping [block]
        "name" [plain @6:3..6:7]: "job" [plain @6:9..6:12]
      "spec" [plain @7:1..7:5]: mapping [block]
        "entrypoint" [plain @8:3..8:13]: "main" [plain @8:15..8:19]
        "arguments" [plain @9:3..9:12]: mapping [block]
          "parameters" [plain @10:5..10:15]: sequence [block]
            - mapping [block]
              "name" [plain @11:9..11:13]: "date" [plain @11:15..11:19]
              "value" [plain @13:9..13:14]: "yesterday" [plain @13:16..13:25]
        "templates" [plain @14:3..14:12]: sequence [block]
          - mapping [block]
            "name" [plain @15:7..15:11]: "main" [plain @15:13..15:17]
            "inputs" [plain @16:7..16:13]: mapping [block]
              "parameters" [plain @17:9..17:19]: sequence [block]
                - mapping [block]
                  "name" [plain @18:13..18:17]: "date" [plain @18:19..18:23]
            "script" [plain @19:7..19:13]: mapping [block]
              "image" [plain @20:9..20:14]: "busybox" [plain @20:16..20:23]
              "command" [plain @21:9..21:16]: sequence [block]
                - "sh" [plain @22:13..22:15]
              "source" [plain @23:9..23:15]: literal [@23:17..29:1]
                "date=\"{{inputs.parameters.date}}\"\n"
                "if [ $date = yesterday ]; then\n"
                "  date=$(date -d yesterday +%Y-%m-%d)\n"
                "fi\n"
                "echo \"run for $date\"\n"

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @30:1..30:11]: "argoproj.io/v1alpha1" [plain @30:13..30:33]
      "kind" [plain @31:1..31:5]: "CronWorkflow" [plain @31:7..31:19]
      "metadata" [plain @32:1..32:9]: mapping [block]
        "name" [plain @33:3..33:7]: "daily-job" [plain @33:9..33:18]
      "spec" [plain @34:1..34:5]: mapping [block]
        "schedules" [plain @36:3..36:12]: sequence [block]
          - "0 2 * * *" [double-quoted @37:7..37:18]
        "workflowSpec" [plain @38:3..38:15]: mapping [block]
          "workflowTemplateRef" [plain @39:5..39:24]: mapping [block]
            "name" [plain @40:7..40:11]: "job" [plain @40:13..40:16]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @44:1..44:11]: "argoproj.io/v1alpha1" [plain @44:13..44:33]
      "kind" [plain @45:1..45:5]: "Workflow" [plain @45:7..45:15]
      "metadata" [plain @46:1..46:9]: mapping [block]
        "name" [plain @47:3..47:7]: "backfill-v1" [plain @47:9..47:20]
        "labels" [plain @48:3..48:9]: mapping [block]
          "workflows.argoproj.io/no-test" [plain @49:5..49:34]: "environment" [double-quoted @49:36..49:49]
      "spec" [plain @50:1..50:5]: mapping [block]
        "entrypoint" [plain @51:3..51:13]: "main" [plain @51:15..51:19]
        "serviceAccountName" [plain @54:3..54:21]: "argo-server" [plain @54:23..54:34]
        "templates" [plain @55:3..55:12]: sequence [block]
          - mapping [block]
            "name" [plain @56:7..56:11]: "main" [plain @56:13..56:17]
            "steps" [plain @57:7..57:12]: sequence [block]
              - sequence [block]
                - mapping [block]
                  "name" [plain @58:13..58:17]: "day" [plain @58:19..58:22]
                  "template" [plain @59:13..59:21]: "create-job" [plain @59:23..59:33]
                  "arguments" [plain @60:13..60:22]: mapping [block]
                    "parameters" [plain @61:15..61:25]: sequence [block]
                      - mapping [block]
                        "name" [plain @62:19..62:23]: "date" [plain @62:25..62:29]
                        "value" [plain @63:19..63:24]: "{{item}}" [double-quoted @63:26..63:36]
                  "withSequence" [plain @64:13..64:25]: mapping [block]
                    "start" [plain @66:15..66:20]: "19" [double-quoted @66:22..66:26]
                    "end" [plain @67:15..67:18]: "21" [double-quoted @67:20..67:24]
                    "format" [plain @68:15..68:21]: "2020-05-%02X" [double-quoted @68:23..68:37]
          - mapping [block]
            "name" [plain @70:7..70:11]: "create-job" [plain @70:13..70:23]
            "inputs" [plain @71:7..71:13]: mapping [block]
              "parameters" [plain @72:9..72:19]: sequence [block]
                - mapping [block]
                  "name" [plain @73:13..73:17]: "date" [plain @73:19..73:23]
            "resource" [plain @74:7..74:15]: mapping [block]
              "action" [plain @75:9..75:15]: "apply" [plain @75:17..75:22]
              "manifest" [plain @76:9..76:17]: literal [@76:19..96:1]
                "apiVersion: argoproj.io/v1alpha1\n"
                "kind: Workflow\n"
                "metadata:\n"
                "  # using a name based on the date prevents re-creating resources,\n"
                "  # making this more robust\n"
                "  name: job-{{inputs.parameters.date}}\n"
                "spec:\n"
                "  entrypoint: main\n"
                "  templates:\n"
                "    - name: main\n"
                "      steps:\n"
                "        - - name: start-job\n"
                "            arguments:\n"
                "              parameters:\n"
                "                - name: date\n"
                "                  value: \"{{inputs.parameters.date}}\"\n"
                "            templateRef:\n"
                "              name: job\n"
                "              template: main\n"

    document 4 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @99:1..99:11]: "argoproj.io/v1alpha1" [plain @99:13..99:33]
      "kind" [plain @100:1..100:5]: "Workflow" [plain @100:7..100:15]
      "metadata" [plain @101:1..101:9]: mapping [block]
        "name" [plain @102:3..102:7]: "backfill-v2" [plain @102:9..102:20]
        "labels" [plain @103:3..103:9]: mapping [block]
          "workflows.argoproj.io/no-test" [plain @104:5..104:34]: "environment" [double-quoted @104:36..104:49]
        "annotations" [plain @105:3..105:14]: mapping [block]
          "workflows.argoproj.io/no-test-reasons" [plain @106:5..106:42]: literal [@106:44..112:1]
            "This is the error, we should fix the service account\n"
            "\n"
            "workflowtaskresults.argoproj.io is forbidden: User\n"
            "\"system:serviceaccount:argo:argo-server\" cannot create resource\n"
            "\"workflowtaskresults\" in API group \"argoproj.io\" in the namespace \"argo\"\n"
      "spec" [plain @112:1..112:5]: mapping [block]
        "entrypoint" [plain @113:3..113:13]: "main" [plain @113:15..113:19]
        "parallelism" [plain @115:3..115:14]: "1" [plain @115:16..115:17]
        "templates" [plain @116:3..116:12]: sequence [block]
          - mapping [block]
            "name" [plain @117:7..117:11]: "main" [plain @117:13..117:17]
            "steps" [plain @118:7..118:12]: sequence [block]
              - sequence [block]
                - mapping [block]
                  "name" [plain @119:13..119:17]: "create-job" [plain @119:19..119:29]
                  "templateRef" [plain @120:13..120:24]: mapping [block]
                    "name" [plain @121:15..121:19]: "job" [plain @121:21..121:24]
                    "template" [plain @122:15..122:23]: "main" [plain @122:25..122:29]
                  "arguments" [plain @123:13..123:22]: mapping [block]
                    "parameters" [plain @124:15..124:25]: sequence [block]
                      - mapping [block]
                        "name" [plain @125:19..125:23]: "date" [plain @125:25..125:29]
                        "value" [plain @126:19..126:24]: "{{item}}" [double-quoted @126:26..126:36]
                  "withSequence" [plain @127:13..127:25]: mapping [block]
                    "start" [plain @129:15..129:20]: "19" [double-quoted @129:22..129:26]
                    "end" [plain @130:15..130:18]: "21" [double-quoted @130:20..130:24]
                    "format" [plain @131:15..131:21]: "2020-05-%02d" [double-quoted @131:23..131:37]
    "#);
}
