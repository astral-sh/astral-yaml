#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-observe-alerts-generated-config-asset-check-failed-slack-1bb32af8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "alert_policies" [plain @3:1..3:15]: sequence [block]
        - mapping [block]
          "alert_targets" [plain @4:3..4:16]: sequence [block]
            - mapping [block]
              "asset_key_target" [plain @5:5..5:21]: mapping [block]
                "asset_key" [plain @6:7..6:16]: sequence [block]
                  - "s3" [plain @7:9..7:11]
                  - "report" [plain @8:9..8:15]
            - mapping [block]
              "asset_group_target" [plain @9:5..9:23]: mapping [block]
                "asset_group" [plain @10:7..10:18]: "transformed" [plain @10:20..10:31]
                "location_name" [plain @11:7..11:20]: "prod" [plain @11:22..11:26]
                "repo_name" [plain @12:7..12:16]: "__repository__" [plain @12:18..12:32]
          "description" [plain @13:3..13:14]: "Sends a Slack message when an asset check fails." [plain @13:16..13:64]
          "event_types" [plain @14:3..14:14]: sequence [block]
            - "ASSET_CHECK_SEVERITY_ERROR" [plain @15:5..15:31]
          "name" [plain @16:3..16:7]: "asset-check-failed-slack" [plain @16:9..16:33]
          "notification_service" [plain @17:3..17:23]: mapping [block]
            "slack" [plain @18:5..18:10]: mapping [block]
              "slack_channel_name" [plain @19:7..19:25]: "notifications" [plain @19:27..19:40]
              "slack_workspace_name" [plain @20:7..20:27]: "hooli" [plain @20:29..20:34]
    "#);
}
