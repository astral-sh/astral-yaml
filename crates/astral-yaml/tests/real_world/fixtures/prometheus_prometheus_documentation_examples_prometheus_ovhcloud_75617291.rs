#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-prometheus-ovhcloud-75617291"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "scrape_configs" [plain @2:1..2:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @3:5..3:13]: "ovhcloud" [single-quoted @3:15..3:25]
          "ovhcloud_sd_configs" [plain @4:5..4:24]: sequence [block]
            - mapping [block]
              "service" [plain @5:9..5:16]: "vps" [plain @5:18..5:21]
              "endpoint" [plain @6:9..6:17]: "ovh-eu" [plain @6:19..6:25]
              "application_key" [plain @7:9..7:24]: "XXX" [plain @7:26..7:29]
              "application_secret" [plain @8:9..8:27]: "XXX" [plain @8:29..8:32]
              "consumer_key" [plain @9:9..9:21]: "XXX" [plain @9:23..9:26]
              "refresh_interval" [plain @10:9..10:25]: "1m" [plain @10:27..10:29]
            - mapping [block]
              "service" [plain @11:9..11:16]: "dedicated_server" [plain @11:18..11:34]
              "endpoint" [plain @12:9..12:17]: "ovh-eu" [plain @12:19..12:25]
              "application_key" [plain @13:9..13:24]: "XXX" [plain @13:26..13:29]
              "application_secret" [plain @14:9..14:27]: "XXX" [plain @14:29..14:32]
              "consumer_key" [plain @15:9..15:21]: "XXX" [plain @15:23..15:26]
              "refresh_interval" [plain @16:9..16:25]: "1m" [plain @16:27..16:29]
    "#);
}
