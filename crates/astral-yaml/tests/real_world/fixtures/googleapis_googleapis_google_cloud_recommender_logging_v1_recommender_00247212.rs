#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-recommender-logging-v1-recommender-00247212"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "recommender.googleapis.com" [plain @3:7..3:33]
      "title" [plain @4:1..4:6]: "Recommender API Logging" [plain @4:8..4:31]
      "types" [plain @6:1..6:6]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.recommender.logging.v1.ActionLog" [plain @7:9..7:54]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.cloud.recommender.logging.v1.InsightActionLog" [plain @8:9..8:61]
    "#);
}
