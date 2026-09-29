#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-videointelligence-v1p1beta1-videointelligence-gapic-009da1f2"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @4:1..4:5]: "com.google.api.codegen.ConfigProto" [plain @4:7..4:41]
      "config_schema_version" [plain @5:1..5:22]: "2.0.0" [plain @5:24..5:29]
      "interfaces" [plain @6:1..6:11]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.videointelligence.v1p1beta1.VideoIntelligenceService" [plain @7:9..7:74]
          "methods" [plain @8:3..8:10]: sequence [block]
            - mapping [block]
              "name" [plain @9:5..9:9]: "AnnotateVideo" [plain @9:11..9:24]
              "long_running" [plain @10:5..10:17]: mapping [block]
                "initial_poll_delay_millis" [plain @11:7..11:32]: "20000" [plain @11:34..11:39]
                "poll_delay_multiplier" [plain @12:7..12:28]: "1.5" [plain @12:30..12:33]
                "max_poll_delay_millis" [plain @13:7..13:28]: "45000" [plain @13:30..13:35]
                "total_poll_timeout_millis" [plain @14:7..14:32]: "86400000" [plain @14:34..14:42]
    "#);
}
