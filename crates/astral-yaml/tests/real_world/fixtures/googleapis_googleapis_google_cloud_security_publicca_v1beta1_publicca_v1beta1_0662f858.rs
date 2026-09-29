#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-security-publicca-v1beta1-publicca-v1beta1-0662f858"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "publicca.googleapis.com" [plain @3:7..3:30]
      "title" [plain @4:1..4:6]: "Public Certificate Authority API" [plain @4:8..4:40]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.security.publicca.v1beta1.PublicCertificateAuthorityService" [plain @7:9..7:81]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: literal [@10:12..15:1]
          "The Public Certificate Authority API may be used to create and manage ACME\n"
          "external account binding keys associated with Google Trust Services'\n"
          "publicly trusted certificate authority."
      "authentication" [plain @15:1..15:15]: mapping [block]
        "rules" [plain @16:3..16:8]: sequence [block]
          - mapping [block]
            "selector" [plain @17:5..17:13]: "google.cloud.security.publicca.v1beta1.PublicCertificateAuthorityService.CreateExternalAccountKey" [plain @17:15..17:112]
            "oauth" [plain @18:5..18:10]: mapping [block]
              "canonical_scopes" [plain @19:7..19:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @19:25..22:1]
      "publishing" [plain @22:1..22:11]: mapping [block]
        "new_issue_uri" [plain @23:3..23:16]: "https://cloud.google.com/certificate-manager/docs/getting-support" [plain @23:18..23:83]
        "documentation_uri" [plain @24:3..24:20]: "https://cloud.google.com/certificate-manager/docs/public-ca" [plain @24:22..24:81]
        "api_short_name" [plain @25:3..25:17]: "publicca" [plain @25:19..25:27]
        "github_label" [plain @26:3..26:15]: "api: publicca" [single-quoted @26:17..26:32]
        "doc_tag_prefix" [plain @27:3..27:17]: "publicca" [plain @27:19..27:27]
        "organization" [plain @28:3..28:15]: "CLOUD" [plain @28:17..28:22]
        "proto_reference_documentation_uri" [plain @29:3..29:36]: "https://cloud.google.com/certificate-manager/docs/reference/public-ca/rpc" [plain @29:38..29:111]
    "#);
}
