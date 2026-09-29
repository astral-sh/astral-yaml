#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-map-reduce-12385d9f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "generateName" [plain @4:3..4:15]: "map-reduce-" [plain @4:17..4:28]
        "annotations" [plain @5:3..5:14]: mapping [block]
          "workflows.argoproj.io/description" [plain @6:5..6:38]: literal [@6:40..11:1]
            "This workflow demonstrates map-reduce using \"key-only\" artifacts.\n"
            "The first task \"split\" produces a number of parts, each in the form of a JSON document, saving it to a bucket.\n"
            "Each \"map\" task then reads those documents, performs a map operation, and writes them out to a new bucket.\n"
            "Finally, \"reduce\" merges all the mapped documents into a final document.\n"
          "workflows.argoproj.io/version" [plain @11:5..11:34]: ">= 3.0.0" [single-quoted @11:36..11:46]
      "spec" [plain @12:1..12:5]: mapping [block]
        "entrypoint" [plain @13:3..13:13]: "main" [plain @13:15..13:19]
        "arguments" [plain @14:3..14:12]: mapping [block]
          "parameters" [plain @15:5..15:15]: sequence [block]
            - mapping [block]
              "name" [plain @16:9..16:13]: "numParts" [plain @16:15..16:23]
              "value" [plain @17:9..17:14]: "4" [double-quoted @17:16..17:19]
        "templates" [plain @18:3..18:12]: sequence [block]
          - mapping [block]
            "name" [plain @19:7..19:11]: "main" [plain @19:13..19:17]
            "dag" [plain @20:7..20:10]: mapping [block]
              "tasks" [plain @21:9..21:14]: sequence [block]
                - mapping [block]
                  "name" [plain @22:13..22:17]: "split" [plain @22:19..22:24]
                  "template" [plain @23:13..23:21]: "split" [plain @23:23..23:28]
                  "arguments" [plain @24:13..24:22]: mapping [block]
                    "parameters" [plain @25:15..25:25]: sequence [block]
                      - mapping [block]
                        "name" [plain @26:19..26:23]: "numParts" [plain @26:25..26:33]
                        "value" [plain @27:19..27:24]: "{{workflow.parameters.numParts}}" [double-quoted @27:26..27:60]
                - mapping [block]
                  "name" [plain @28:13..28:17]: "map" [plain @28:19..28:22]
                  "template" [plain @29:13..29:21]: "map" [plain @29:23..29:26]
                  "arguments" [plain @30:13..30:22]: mapping [block]
                    "parameters" [plain @31:15..31:25]: sequence [block]
                      - mapping [block]
                        "name" [plain @32:19..32:23]: "partId" [plain @32:25..32:31]
                        "value" [plain @33:19..33:24]: "{{item}}" [single-quoted @33:26..33:36]
                    "artifacts" [plain @34:15..34:24]: sequence [block]
                      - mapping [block]
                        "name" [plain @35:19..35:23]: "part" [plain @35:25..35:29]
                        "s3" [plain @36:19..36:21]: mapping [block]
                          "key" [plain @37:21..37:24]: "{{workflow.name}}/parts/{{item}}.json" [double-quoted @37:26..37:65]
                  "depends" [plain @38:13..38:20]: "split" [double-quoted @38:22..38:29]
                  "withParam" [plain @39:13..39:22]: "{{tasks.split.outputs.result}}" [single-quoted @39:24..39:56]
                - mapping [block]
                  "name" [plain @40:13..40:17]: "reduce" [plain @40:19..40:25]
                  "template" [plain @41:13..41:21]: "reduce" [plain @41:23..41:29]
                  "depends" [plain @42:13..42:20]: "map" [double-quoted @42:22..42:27]
          - mapping [block]
            "name" [plain @47:7..47:11]: "split" [plain @47:13..47:18]
            "inputs" [plain @48:7..48:13]: mapping [block]
              "parameters" [plain @49:9..49:19]: sequence [block]
                - mapping [block]
                  "name" [plain @50:13..50:17]: "numParts" [plain @50:19..50:27]
            "script" [plain @51:7..51:13]: mapping [block]
              "image" [plain @52:9..52:14]: "python:alpine3.23" [plain @52:16..52:33]
              "command" [plain @53:9..53:16]: sequence [block]
                - "python" [plain @54:13..54:19]
              "source" [plain @55:9..55:15]: literal [@55:17..65:1]
                "import json\n"
                "import os\n"
                "import sys\n"
                "os.mkdir(\"/mnt/out\")\n"
                "partIds = list(map(lambda x: str(x), range({{inputs.parameters.numParts}})))\n"
                "for i, partId in enumerate(partIds, start=1):\n"
                "  with open(\"/mnt/out/\" + partId + \".json\", \"w\") as f:\n"
                "    json.dump({\"foo\": i}, f)\n"
                "json.dump(partIds, sys.stdout)\n"
            "outputs" [plain @65:7..65:14]: mapping [block]
              "artifacts" [plain @66:9..66:18]: sequence [block]
                - mapping [block]
                  "name" [plain @67:13..67:17]: "parts" [plain @67:19..67:24]
                  "path" [plain @68:13..68:17]: "/mnt/out" [plain @68:19..68:27]
                  "archive" [plain @69:13..69:20]: mapping [block]
                    "none" [plain @70:15..70:19]: mapping [flow]
                  "s3" [plain @71:13..71:15]: mapping [block]
                    "key" [plain @72:15..72:18]: "{{workflow.name}}/parts/" [double-quoted @72:20..72:46]
          - mapping [block]
            "name" [plain @75:7..75:11]: "map" [plain @75:13..75:16]
            "inputs" [plain @76:7..76:13]: mapping [block]
              "parameters" [plain @77:9..77:19]: sequence [block]
                - mapping [block]
                  "name" [plain @78:13..78:17]: "partId" [plain @78:19..78:25]
              "artifacts" [plain @79:9..79:18]: sequence [block]
                - mapping [block]
                  "name" [plain @80:13..80:17]: "part" [plain @80:19..80:23]
                  "path" [plain @81:13..81:17]: "/mnt/in/part.json" [plain @81:19..81:36]
            "script" [plain @82:7..82:13]: mapping [block]
              "image" [plain @83:9..83:14]: "python:alpine3.23" [plain @83:16..83:33]
              "command" [plain @84:9..84:16]: sequence [block]
                - "python" [plain @85:13..85:19]
              "source" [plain @86:9..86:15]: literal [@86:17..95:1]
                "import json\n"
                "import os\n"
                "import sys\n"
                "os.mkdir(\"/mnt/out\")\n"
                "with open(\"/mnt/in/part.json\") as f:\n"
                "  part = json.load(f)\n"
                "with open(\"/mnt/out/part.json\", \"w\") as f:\n"
                "  json.dump({\"bar\": part[\"foo\"] * 2}, f)\n"
            "outputs" [plain @95:7..95:14]: mapping [block]
              "artifacts" [plain @96:9..96:18]: sequence [block]
                - mapping [block]
                  "name" [plain @97:13..97:17]: "part" [plain @97:19..97:23]
                  "path" [plain @98:13..98:17]: "/mnt/out/part.json" [plain @98:19..98:37]
                  "archive" [plain @99:13..99:20]: mapping [block]
                    "none" [plain @100:15..100:19]: mapping [flow]
                  "s3" [plain @101:13..101:15]: mapping [block]
                    "key" [plain @102:15..102:18]: "{{workflow.name}}/results/{{inputs.parameters.partId}}.json" [double-quoted @102:20..102:81]
          - mapping [block]
            "name" [plain @104:7..104:11]: "reduce" [plain @104:13..104:19]
            "inputs" [plain @105:7..105:13]: mapping [block]
              "artifacts" [plain @106:9..106:18]: sequence [block]
                - mapping [block]
                  "name" [plain @107:13..107:17]: "results" [plain @107:19..107:26]
                  "path" [plain @108:13..108:17]: "/mnt/in" [plain @108:19..108:26]
                  "s3" [plain @109:13..109:15]: mapping [block]
                    "key" [plain @110:15..110:18]: "{{workflow.name}}/results" [double-quoted @110:20..110:47]
            "script" [plain @111:7..111:13]: mapping [block]
              "image" [plain @112:9..112:14]: "python:alpine3.23" [plain @112:16..112:33]
              "command" [plain @113:9..113:16]: sequence [block]
                - "python" [plain @114:13..114:19]
              "source" [plain @115:9..115:15]: literal [@115:17..126:1]
                "import json\n"
                "import os\n"
                "import sys\n"
                "total = 0\n"
                "os.mkdir(\"/mnt/out\")\n"
                "for f in list(map(lambda x: open(\"/mnt/in/\" + x), os.listdir(\"/mnt/in\"))):\n"
                "  result = json.load(f)\n"
                "  total = total + result[\"bar\"]\n"
                "with open(\"/mnt/out/total.json\" , \"w\") as f:\n"
                "  json.dump({\"total\": total}, f)\n"
            "outputs" [plain @126:7..126:14]: mapping [block]
              "artifacts" [plain @127:9..127:18]: sequence [block]
                - mapping [block]
                  "name" [plain @128:13..128:17]: "total" [plain @128:19..128:24]
                  "path" [plain @129:13..129:17]: "/mnt/out/total.json" [plain @129:19..129:38]
                  "archive" [plain @130:13..130:20]: mapping [block]
                    "none" [plain @131:15..131:19]: mapping [flow]
                  "s3" [plain @132:13..132:15]: mapping [block]
                    "key" [plain @133:15..133:18]: "{{workflow.name}}/total.json" [double-quoted @133:20..133:50]
    "#);
}
