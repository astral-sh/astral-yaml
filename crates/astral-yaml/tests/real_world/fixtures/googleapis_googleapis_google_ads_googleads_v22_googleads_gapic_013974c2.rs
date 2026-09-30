#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-ads-googleads-v22-googleads-gapic-013974c2"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "com.google.api.codegen.ConfigProto" [plain @1:7..1:41]
      "config_schema_version" [plain @2:1..2:22]: "2.0.0" [plain @2:24..2:29]
      "language_settings" [plain @3:1..3:18]: mapping [block]
        "java" [plain @4:3..4:7]: mapping [block]
          "package_name" [plain @5:5..5:17]: "com.google.ads.googleads.v22.services" [plain @5:19..5:56]
        "php" [plain @6:3..6:6]: mapping [block]
          "package_name" [plain @7:5..7:17]: "Google\\Ads\\GoogleAds\\V22\\Services" [plain @7:19..7:52]
      "interfaces" [plain @8:1..8:11]: sequence [block]
        - mapping [block]
          "name" [plain @9:3..9:7]: "google.ads.googleads.v22.services.OfflineUserDataJobService" [plain @9:9..9:68]
          "methods" [plain @10:3..10:10]: sequence [block]
            - mapping [block]
              "name" [plain @11:5..11:9]: "RunOfflineUserDataJob" [plain @11:11..11:32]
              "long_running" [plain @12:5..12:17]: mapping [block]
                "initial_poll_delay_millis" [plain @13:7..13:32]: "300000" [plain @13:34..13:40]
                "max_poll_delay_millis" [plain @14:7..14:28]: "3600000" [plain @14:30..14:37]
                "poll_delay_multiplier" [plain @15:7..15:28]: "1.25" [plain @15:30..15:34]
                "total_poll_timeout_millis" [plain @16:7..16:32]: "43200000" [plain @16:34..16:42]
    "#);
}
