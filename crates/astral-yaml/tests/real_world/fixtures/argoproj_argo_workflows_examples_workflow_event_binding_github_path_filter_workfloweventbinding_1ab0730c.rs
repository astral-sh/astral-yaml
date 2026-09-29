#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-workflow-event-binding-github-path-filter-workfloweventbinding-1ab0730c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @6:1..6:11]: "argoproj.io/v1alpha1" [plain @6:13..6:33]
      "kind" [plain @7:1..7:5]: "WorkflowEventBinding" [plain @7:7..7:27]
      "metadata" [plain @8:1..8:9]: mapping [block]
        "name" [plain @9:3..9:7]: "github-path-filter" [plain @9:9..9:27]
      "spec" [plain @10:1..10:5]: mapping [block]
        "event" [plain @11:3..11:8]: mapping [block]
          "selector" [plain @12:5..12:13]: literal [@12:15..40:1]
            "metadata[\"x-github-event\"] == [\"push\"]\n"
            "\n"
            "// The discriminator is the last subpath of URL path\n"
            "&& discriminator == \"github-path-filter\"\n"
            "\n"
            "// for each of these file paths\n"
            "&& any([ \"subpath/A/\", \"subpath/B/\" ], {\n"
            "    let prefix = #;\n"
            "\n"
            "    // loop over every commit\n"
            "    any(payload.commits ?? [], {\n"
            "      let commit = #;\n"
            "\n"
            "      // loop over each file action in the commit\n"
            "      any([\"added\", \"removed\", \"modified\"], {\n"
            "        let file_action = #;\n"
            "\n"
            "        // Loop over each of the files in the file action\n"
            "        any(commit[file_action] ?? [], {\n"
            "          let filepath = #;\n"
            "\n"
            "          // return true if the given file is in our subpath\n"
            "          hasPrefix(filepath, prefix)\n"
            "        })\n"
            "      })\n"
            "    })\n"
            "  })\n"
        "submit" [plain @40:3..40:9]: mapping [block]
          "workflowTemplateRef" [plain @41:5..41:24]: mapping [block]
            "name" [plain @42:7..42:11]: "github-path-filter" [plain @42:13..42:31]
          "arguments" [plain @43:5..43:14]: mapping [block]
            "parameters" [plain @44:7..44:17]: sequence [block]
              - mapping [block]
                "name" [plain @45:11..45:15]: "files" [plain @45:17..45:22]
                "valueFrom" [plain @46:11..46:20]: mapping [block]
                  "event" [plain @47:13..47:18]: literal [@47:20..58:1]
                    "// Use the reduce function to collect all files affeccted by commits in this push\n"
                    "reduce(\n"
                    "  payload.commits ?? [],\n"
                    "  {\n"
                    "    let commit = #;\n"
                    "    concat(#acc, commit[\"added\"], commit[\"removed\"], commit[\"modified\"])\n"
                    "  },\n"
                    "  []\n"
                    ")\n"
    "#);
}
