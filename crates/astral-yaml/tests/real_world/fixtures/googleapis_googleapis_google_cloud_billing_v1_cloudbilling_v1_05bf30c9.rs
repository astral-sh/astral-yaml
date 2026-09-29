#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-billing-v1-cloudbilling-v1-05bf30c9"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "cloudbilling.googleapis.com" [plain @3:7..3:34]
      "title" [plain @4:1..4:6]: "Cloud Billing API" [plain @4:8..4:25]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.billing.v1.CloudBilling" [plain @7:9..7:45]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.cloud.billing.v1.CloudCatalog" [plain @8:9..8:45]
      "documentation" [plain @10:1..10:14]: mapping [block]
        "summary" [plain @11:3..11:10]: literal [@11:12..15:1]
          "Allows developers to manage billing for their Google Cloud Platform\n"
          "projects     programmatically."
      "authentication" [plain @15:1..15:15]: mapping [block]
        "rules" [plain @16:3..16:8]: sequence [block]
          - mapping [block]
            "selector" [plain @17:5..17:13]: "google.cloud.billing.v1.CloudBilling.*" [single-quoted @17:15..17:55]
            "oauth" [plain @18:5..18:10]: mapping [block]
              "canonical_scopes" [plain @19:7..19:23]: literal [@19:25..23:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-billing.readonly,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @23:5..23:13]: "google.cloud.billing.v1.CloudBilling.CreateBillingAccount" [plain @23:15..23:72]
            "oauth" [plain @24:5..24:10]: mapping [block]
              "canonical_scopes" [plain @25:7..25:23]: literal [@25:25..28:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @28:5..28:13]: "google.cloud.billing.v1.CloudBilling.MoveBillingAccount" [plain @28:15..28:70]
            "oauth" [plain @29:5..29:10]: mapping [block]
              "canonical_scopes" [plain @30:7..30:23]: literal [@30:25..33:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @33:5..33:13]: "google.cloud.billing.v1.CloudBilling.SetIamPolicy" [plain @33:15..33:64]
            "oauth" [plain @34:5..34:10]: mapping [block]
              "canonical_scopes" [plain @35:7..35:23]: literal [@35:25..38:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @38:5..38:13]: "google.cloud.billing.v1.CloudBilling.UpdateBillingAccount" [plain @38:15..38:72]
            "oauth" [plain @39:5..39:10]: mapping [block]
              "canonical_scopes" [plain @40:7..40:23]: literal [@40:25..43:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @43:5..43:13]: "google.cloud.billing.v1.CloudBilling.UpdateProjectBillingInfo" [plain @43:15..43:76]
            "oauth" [plain @44:5..44:10]: mapping [block]
              "canonical_scopes" [plain @45:7..45:23]: literal [@45:25..48:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @48:5..48:13]: "google.cloud.billing.v1.CloudCatalog.ListServices" [plain @48:15..48:64]
            "oauth" [plain @49:5..49:10]: mapping [block]
              "canonical_scopes" [plain @50:7..50:23]: literal [@50:25..54:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-billing.readonly,\n"
                "https://www.googleapis.com/auth/cloud-platform"
          - mapping [block]
            "selector" [plain @54:5..54:13]: "google.cloud.billing.v1.CloudCatalog.ListSkus" [plain @54:15..54:60]
            "oauth" [plain @55:5..55:10]: mapping [block]
              "canonical_scopes" [plain @56:7..56:23]: literal [@56:25..60:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-billing.readonly,\n"
                "https://www.googleapis.com/auth/cloud-platform"
    "#);
}
