#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-devtools-containeranalysis-v1beta1-containeranalysis-v1beta1-016c7d3a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "containeranalysis.googleapis.com" [plain @3:7..3:39]
      "title" [plain @4:1..4:6]: "Container Analysis API" [plain @4:8..4:30]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.devtools.containeranalysis.v1beta1.ContainerAnalysisV1Beta1" [plain @7:9..7:75]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: literal [@10:12..19:1]
          "This API is a prerequisite for leveraging Artifact Analysis\n"
          "scanning capabilities in both Artifact Registry and with Advanced\n"
          "Vulnerability Insights (runtime scanning) in GKE.\n"
          "\n"
          "In addition, the Container Analysis API is an implementation of the\n"
          "Grafeas API, which enables storing, querying, and retrieval of critical\n"
          "metadata\n"
          "about all of your software artifacts."
        "overview" [plain @19:3..19:11]: literal [@19:13..23:1]
          "The Container Analysis API allows you to store and retrieve metadata for a\n"
          "container resource."
      "authentication" [plain @23:1..23:15]: mapping [block]
        "rules" [plain @24:3..24:8]: sequence [block]
          - mapping [block]
            "selector" [plain @25:5..25:13]: "google.devtools.containeranalysis.v1beta1.ContainerAnalysisV1Beta1.*" [single-quoted @25:15..25:85]
            "oauth" [plain @26:5..26:10]: mapping [block]
              "canonical_scopes" [plain @27:7..27:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @27:25..30:1]
      "publishing" [plain @30:1..30:11]: mapping [block]
        "library_settings" [plain @31:3..31:19]: sequence [block]
          - mapping [block]
            "version" [plain @32:5..32:12]: "google.devtools.containeranalysis.v1beta1" [plain @32:14..32:55]
            "java_settings" [plain @33:5..33:18]: mapping [block]
              "library_package" [plain @34:7..34:22]: "com.google.cloud.devtools.containeranalysis.v1beta1" [plain @34:24..34:75]
    "#);
}
