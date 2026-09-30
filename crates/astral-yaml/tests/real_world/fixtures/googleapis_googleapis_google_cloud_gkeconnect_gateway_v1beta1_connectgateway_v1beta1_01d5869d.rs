#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-gkeconnect-gateway-v1beta1-connectgateway-v1beta1-01d5869d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "connectgateway.googleapis.com" [plain @3:7..3:36]
      "title" [plain @4:1..4:6]: "Connect Gateway API" [plain @4:8..4:27]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.gkeconnect.gateway.v1beta1.GatewayControl" [plain @7:9..7:63]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: literal [@10:12..14:1]
          "The Connect Gateway service allows connectivity from external parties to\n"
          "connected Kubernetes clusters."
      "authentication" [plain @14:1..14:15]: mapping [block]
        "rules" [plain @15:3..15:8]: sequence [block]
          - mapping [block]
            "selector" [plain @16:5..16:13]: "google.cloud.gkeconnect.gateway.v1beta1.GatewayControl.GenerateCredentials" [plain @16:15..16:89]
            "oauth" [plain @17:5..17:10]: mapping [block]
              "canonical_scopes" [plain @18:7..18:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @18:25..21:1]
      "publishing" [plain @21:1..21:11]: mapping [block]
        "new_issue_uri" [plain @22:3..22:16]: "https://issuetracker.google.com/issues/new?component=1618911" [plain @22:18..22:78]
        "documentation_uri" [plain @23:3..23:20]: "https://cloud.google.com/kubernetes-engine/enterprise/multicluster-management/gateway" [plain @23:22..23:107]
        "api_short_name" [plain @24:3..24:17]: "connectgateway" [plain @24:19..24:33]
        "github_label" [plain @25:3..25:15]: "api: connectgateway" [single-quoted @25:17..25:38]
        "doc_tag_prefix" [plain @26:3..26:17]: "connectgateway" [plain @26:19..26:33]
        "organization" [plain @27:3..27:15]: "CLOUD" [plain @27:17..27:22]
    "#);
}
