#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-asset-v1p2beta1-cloudasset-v1p2beta1-045b7609"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "cloudasset.googleapis.com" [plain @3:7..3:32]
      "title" [plain @4:1..4:6]: "Cloud Asset API" [plain @4:8..4:23]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.asset.v1p2beta1.AssetService" [plain @7:9..7:50]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.longrunning.Operations" [plain @8:9..8:38]
      "documentation" [plain @10:1..10:14]: mapping [block]
        "summary" [plain @11:3..11:10]: literal [@11:12..14:1]
          "The Cloud Asset API manages the history and inventory of Google Cloud\n"
          "resources."
        "overview" [plain @14:3..14:11]: literal [@14:13..25:1]
          "# Cloud Asset API\n"
          "\n"
          "The Cloud Asset API keeps a history of Google Cloud asset metadata, and\n"
          "allows Google Cloud users to download a dump of all asset metadata for the\n"
          "resource types listed below within an organization or a project at a given\n"
          "timestamp.\n"
          "\n"
          "Read more documents here:\n"
          "https://cloud.google.com/asset-inventory/docs"
      "http" [plain @25:1..25:5]: mapping [block]
        "rules" [plain @26:3..26:8]: sequence [block]
          - mapping [block]
            "selector" [plain @27:5..27:13]: "google.longrunning.Operations.GetOperation" [plain @27:15..27:57]
            "get" [plain @28:5..28:8]: "/v1p2beta1/{name=*/*/operations/*/**}" [single-quoted @28:10..28:49]
      "authentication" [plain @30:1..30:15]: mapping [block]
        "rules" [plain @31:3..31:8]: sequence [block]
          - mapping [block]
            "selector" [plain @32:5..32:13]: "google.cloud.asset.v1p2beta1.AssetService.*" [single-quoted @32:15..32:60]
            "oauth" [plain @33:5..33:10]: mapping [block]
              "canonical_scopes" [plain @34:7..34:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @34:25..36:1]
          - mapping [block]
            "selector" [plain @36:5..36:13]: "google.longrunning.Operations.GetOperation" [plain @36:15..36:57]
            "oauth" [plain @37:5..37:10]: mapping [block]
              "canonical_scopes" [plain @38:7..38:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @38:25..40:1]
    "##);
}
