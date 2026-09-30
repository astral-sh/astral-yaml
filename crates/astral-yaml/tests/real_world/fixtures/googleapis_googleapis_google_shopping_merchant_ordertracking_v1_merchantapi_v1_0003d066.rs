#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-shopping-merchant-ordertracking-v1-merchantapi-v1-0003d066"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "merchantapi.googleapis.com" [plain @3:7..3:33]
      "title" [plain @4:1..4:6]: "Merchant API" [plain @4:8..4:20]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.shopping.merchant.ordertracking.v1.OrderTrackingSignalsService" [plain @7:9..7:78]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: "Programmatically manage your Merchant Center Accounts." [plain @10:12..10:66]
        "overview" [plain @11:3..11:11]: literal [@11:13..28:1]
          "Merchant API consists of multiple Sub-APIs. Accounts Sub-API: Enables you\n"
          "to programmatically manage your accounts. Conversions Sub-API: Enables you\n"
          "to programmatically manage your conversion sources for a merchant account.\n"
          "Datasources Sub-API: Enables you to programmatically manage your\n"
          "datasources. Inventories Sub-API: This bundle enables you to\n"
          "programmatically manage your local and regional inventories. Local Feeds\n"
          "Partnerships Sub-API: This bundle enables LFP partners to submit local\n"
          "inventories for a merchant. Notifications Sub-API: This bundle enables you\n"
          "to programmatically manage your notification subscriptions. Products\n"
          "Sub-API: This bundle enables you to programmatically manage your products.\n"
          "Promotions Sub-API: This bundle enables you to programmatically manage\n"
          "your promotions for products. Quota Sub-API: This bundle enables you to\n"
          "list your quotas for all APIs you are using. Reports Sub-API: This bundle\n"
          "enables you to programmatically retrieve reports and insights about\n"
          "products, their performance and their competitive environment."
      "authentication" [plain @28:1..28:15]: mapping [block]
        "rules" [plain @29:3..29:8]: sequence [block]
          - mapping [block]
            "selector" [plain @30:5..30:13]: "google.shopping.merchant.ordertracking.v1.OrderTrackingSignalsService.CreateOrderTrackingSignal" [plain @30:15..30:110]
            "oauth" [plain @31:5..31:10]: mapping [block]
              "canonical_scopes" [plain @32:7..32:23]: "https://www.googleapis.com/auth/content" [literal @32:25..35:1]
      "publishing" [plain @35:1..35:11]: mapping [block]
        "new_issue_uri" [plain @36:3..36:16]: "https://issuetracker.google.com/issues/new?component=171084&template=555201" [plain @36:18..36:93]
        "documentation_uri" [plain @37:3..37:20]: "https://developers.google.com/merchant/api" [plain @37:22..37:64]
        "api_short_name" [plain @38:3..38:17]: "merchantapi" [plain @38:19..38:30]
        "github_label" [plain @39:3..39:15]: "api: merchantapi" [single-quoted @39:17..39:35]
        "doc_tag_prefix" [plain @40:3..40:17]: "merchantapi" [plain @40:19..40:30]
        "organization" [plain @41:3..41:15]: "SHOPPING" [plain @41:17..41:25]
        "library_settings" [plain @42:3..42:19]: sequence [block]
          - mapping [block]
            "version" [plain @43:5..43:12]: "google.shopping.merchant.ordertracking.v1" [plain @43:14..43:55]
            "launch_stage" [plain @44:5..44:17]: "GA" [plain @44:19..44:21]
            "java_settings" [plain @45:5..45:18]: mapping [block]
              "common" [plain @46:7..46:13]: mapping [block]
                "destinations" [plain @47:9..47:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @48:11..48:26]
            "php_settings" [plain @49:5..49:17]: mapping [block]
              "common" [plain @50:7..50:13]: mapping [block]
                "destinations" [plain @51:9..51:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @52:11..52:26]
            "python_settings" [plain @53:5..53:20]: mapping [block]
              "common" [plain @54:7..54:13]: mapping [block]
                "destinations" [plain @55:9..55:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @56:11..56:26]
            "node_settings" [plain @57:5..57:18]: mapping [block]
              "common" [plain @58:7..58:13]: mapping [block]
                "destinations" [plain @59:9..59:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @60:11..60:26]
            "dotnet_settings" [plain @61:5..61:20]: mapping [block]
              "common" [plain @62:7..62:13]: mapping [block]
                "destinations" [plain @63:9..63:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @64:11..64:26]
            "ruby_settings" [plain @65:5..65:18]: mapping [block]
              "common" [plain @66:7..66:13]: mapping [block]
                "destinations" [plain @67:9..67:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @68:11..68:26]
            "go_settings" [plain @69:5..69:16]: mapping [block]
              "common" [plain @70:7..70:13]: mapping [block]
                "destinations" [plain @71:9..71:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @72:11..72:26]
    "#);
}
