#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-billing-budgets-v1-billingbudgets-072a3588"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "billingbudgets.googleapis.com" [plain @3:7..3:36]
      "title" [plain @4:1..4:6]: "Cloud Billing Budget API" [plain @4:8..4:32]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.billing.budgets.v1.BudgetService" [plain @7:9..7:54]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: literal [@10:12..15:1]
          "The Cloud Billing Budget API stores Cloud Billing budgets, which define a\n"
          "budget plan and the rules to execute as spend is tracked against that\n"
          "plan."
      "authentication" [plain @15:1..15:15]: mapping [block]
        "rules" [plain @16:3..16:8]: sequence [block]
          - mapping [block]
            "selector" [plain @17:5..17:13]: "google.cloud.billing.budgets.v1.BudgetService.*" [single-quoted @17:15..17:64]
            "oauth" [plain @18:5..18:10]: mapping [block]
              "canonical_scopes" [plain @19:7..19:23]: literal [@19:25..22:1]
                "https://www.googleapis.com/auth/cloud-billing,\n"
                "https://www.googleapis.com/auth/cloud-platform"
    "#);
}
