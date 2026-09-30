#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-artifacts-workflowtemplate-15f86354"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "WorkflowTemplate" [plain @2:7..2:23]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "artifacts" [plain @4:9..4:18]
        "annotations" [plain @5:3..5:14]: mapping [block]
          "workflows.argoproj.io/description" [plain @6:5..6:38]: literal [@6:40..8:1]
            "This example shows how to produce different types of artifact.\n"
      "spec" [plain @8:1..8:5]: mapping [block]
        "entrypoint" [plain @9:3..9:13]: "main" [plain @9:15..9:19]
        "templates" [plain @10:3..10:12]: sequence [block]
          - mapping [block]
            "name" [plain @11:7..11:11]: "main" [plain @11:13..11:17]
            "volumes" [plain @12:7..12:14]: sequence [block]
              - mapping [block]
                "name" [plain @13:11..13:15]: "in" [plain @13:17..13:19]
                "emptyDir" [plain @14:11..14:19]: mapping [flow]
              - mapping [block]
                "name" [plain @15:11..15:15]: "out" [plain @15:17..15:20]
                "emptyDir" [plain @16:11..16:19]: mapping [flow]
            "inputs" [plain @17:7..17:13]: mapping [block]
              "artifacts" [plain @18:9..18:18]: sequence [block]
                - mapping [block]
                  "name" [plain @19:13..19:17]: "temps" [plain @19:19..19:24]
                  "path" [plain @20:13..20:17]: "/in/annual.csv" [plain @20:19..20:33]
                  "http" [plain @21:13..21:17]: mapping [block]
                    "url" [plain @22:15..22:18]: "https://datahub.io/core/global-temp/r/annual.csv" [plain @22:20..22:68]
            "containerSet" [plain @23:7..23:19]: mapping [block]
              "volumeMounts" [plain @24:9..24:21]: sequence [block]
                - mapping [block]
                  "mountPath" [plain @25:13..25:22]: "/in" [plain @25:24..25:27]
                  "name" [plain @26:13..26:17]: "in" [plain @26:19..26:21]
                - mapping [block]
                  "mountPath" [plain @27:13..27:22]: "/out" [plain @27:24..27:28]
                  "name" [plain @28:13..28:17]: "out" [plain @28:19..28:22]
              "containers" [plain @29:9..29:19]: sequence [block]
                - mapping [block]
                  "name" [plain @30:13..30:17]: "setup" [plain @30:19..30:24]
                  "image" [plain @31:13..31:18]: "argoproj/argosay:v2" [plain @31:20..31:39]
                  "command" [plain @32:13..32:20]: sequence [block]
                    - "sh" [plain @33:17..33:19]
                    - "-c" [plain @34:17..34:19]
                  "args" [plain @35:13..35:17]: sequence [block]
                    - literal [@36:17..38:1]
                      "mkdir -p /out/assets\n"
                - mapping [block]
                  "name" [plain @38:13..38:17]: "gnuplot" [plain @38:19..38:26]
                  "image" [plain @39:13..39:18]: "remuslazar/gnuplot" [plain @39:20..39:38]
                  "dependencies" [plain @40:13..40:25]: sequence [block]
                    - "setup" [plain @41:17..41:22]
                  "args" [plain @42:13..42:17]: sequence [block]
                    - "-e" [plain @43:17..43:19]
                    - literal [@44:17..51:1]
                      "set xlabel 'Year'; set ylabel 'Mean';\n"
                      "set grid;\n"
                      "set datafile separator ',';\n"
                      "set term png size 600,400;\n"
                      "set output '/out/assets/global-temp.png';\n"
                      "plot '/in/annual.csv' every 2::0 skip 1 using 2:3 title 'Global Temperature' with lines linewidth 2;\n"
                - mapping [block]
                  "name" [plain @51:13..51:17]: "main" [plain @51:19..51:23]
                  "image" [plain @52:13..52:18]: "argoproj/argosay:v2" [plain @52:20..52:39]
                  "dependencies" [plain @53:13..53:25]: sequence [block]
                    - "setup" [plain @54:17..54:22]
                  "command" [plain @55:13..55:20]: sequence [block]
                    - "sh" [plain @56:17..56:19]
                    - "-c" [plain @57:17..57:19]
                  "args" [plain @58:13..58:17]: sequence [block]
                    - literal [@59:17..89:1]
                      "cowsay \"hello world\" > /out/hello.txt\n"
                      "\n"
                      "cat > /out/hello.json <<EOF\n"
                      "{\"hello\": {\"world\": true}} \n"
                      "EOF\n"
                      "\n"
                      "echo '* {font-family: sans-serif}' > /out/assets/styles.css\n"
                      "\n"
                      "cat > /out/index.html <<EOF\n"
                      "<html>\n"
                      "  <head>\n"
                      "    <link rel='stylesheet' href='assets/styles.css' type='text/css'/>\n"
                      "  </head>\n"
                      "  <body>\n"
                      "    <h1>Global Temperature</h1>\n"
                      "    <img src='assets/global-temp.png'/>\n"
                      "  </body>\n"
                      "</html>\n"
                      "EOF\n"
                      "\n"
                      "cat > /out/malicious.html <<EOF\n"
                      "<html>\n"
                      "  <body>\n"
                      "    <script>alert(1)</script>\n"
                      "    <p>This page attempts to run a script that shows an alert, but the Argo Server UI Content-Security-Policy will prevent that.</p>\n"
                      "    <p>To check, open your Web Console and see that \"Blocked script execution ... because the document's frame is sandboxed.\" (or similar) is printed.</p>\n"
                      "  </body>\n"
                      "</html>\n"
                      "EOF\n"
            "outputs" [plain @89:7..89:14]: mapping [block]
              "artifacts" [plain @90:9..90:18]: sequence [block]
                - mapping [block]
                  "name" [plain @92:13..92:17]: "text-file" [plain @92:19..92:28]
                  "path" [plain @93:13..93:17]: "/out/hello.txt" [plain @93:19..93:33]
                  "s3" [plain @94:13..94:15]: mapping [block]
                    "key" [plain @95:15..95:18]: "hello.txt" [plain @95:20..95:29]
                  "archive" [plain @96:13..96:20]: mapping [block]
                    "none" [plain @97:15..97:19]: mapping [flow]
                - mapping [block]
                  "name" [plain @99:13..99:17]: "json-file" [plain @99:19..99:28]
                  "path" [plain @100:13..100:17]: "/out/hello.json" [plain @100:19..100:34]
                  "s3" [plain @101:13..101:15]: mapping [block]
                    "key" [plain @102:15..102:18]: "hello.json" [plain @102:20..102:30]
                  "archive" [plain @103:13..103:20]: mapping [block]
                    "none" [plain @104:15..104:19]: mapping [flow]
                - mapping [block]
                  "name" [plain @106:13..106:17]: "css-file" [plain @106:19..106:27]
                  "path" [plain @107:13..107:17]: "/out/assets/styles.css" [plain @107:19..107:41]
                  "s3" [plain @108:13..108:15]: mapping [block]
                    "key" [plain @109:15..109:18]: "styles.css" [plain @109:20..109:30]
                  "archive" [plain @110:13..110:20]: mapping [block]
                    "none" [plain @111:15..111:19]: mapping [flow]
                - mapping [block]
                  "name" [plain @113:13..113:17]: "malicious-file" [plain @113:19..113:33]
                  "path" [plain @114:13..114:17]: "/out/malicious.html" [plain @114:19..114:38]
                  "s3" [plain @115:13..115:15]: mapping [block]
                    "key" [plain @116:15..116:18]: "malicious.html" [plain @116:20..116:34]
                  "archive" [plain @117:13..117:20]: mapping [block]
                    "none" [plain @118:15..118:19]: mapping [flow]
                - mapping [block]
                  "name" [plain @120:13..120:17]: "report" [plain @120:19..120:25]
                  "path" [plain @121:13..121:17]: "/out" [plain @121:19..121:23]
                  "s3" [plain @122:13..122:15]: mapping [block]
                    "key" [plain @123:15..123:18]: "report/" [plain @123:20..123:27]
                  "archive" [plain @124:13..124:20]: mapping [block]
                    "none" [plain @125:15..125:19]: mapping [flow]
                - mapping [block]
                  "name" [plain @127:13..127:17]: "tgz-file" [plain @127:19..127:27]
                  "path" [plain @128:13..128:17]: "/out/hello.txt" [plain @128:19..128:33]
                  "s3" [plain @129:13..129:15]: mapping [block]
                    "key" [plain @130:15..130:18]: "file.tgz" [plain @130:20..130:28]
                - mapping [block]
                  "name" [plain @132:13..132:17]: "tgz-dir" [plain @132:19..132:26]
                  "path" [plain @133:13..133:17]: "/out" [plain @133:19..133:23]
                  "s3" [plain @134:13..134:15]: mapping [block]
                    "key" [plain @135:15..135:18]: "dir.tgz" [plain @135:20..135:27]
    "#);
}
