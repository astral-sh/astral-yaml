#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-apigeeconnect-v1-apigeeconnect-v1-0759b099"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "apigeeconnect.googleapis.com" [plain @3:7..3:35]
      "title" [plain @4:1..4:6]: "Apigee Connect API" [plain @4:8..4:26]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.apigeeconnect.v1.ConnectionService" [plain @7:9..7:56]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.cloud.apigeeconnect.v1.Tether" [plain @8:9..8:45]
      "authentication" [plain @10:1..10:15]: mapping [block]
        "rules" [plain @11:3..11:8]: sequence [block]
          - mapping [block]
            "selector" [plain @12:5..12:13]: "google.cloud.apigeeconnect.v1.ConnectionService.ListConnections" [plain @12:15..12:78]
            "oauth" [plain @13:5..13:10]: mapping [block]
              "canonical_scopes" [plain @14:7..14:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @14:25..16:1]
          - mapping [block]
            "selector" [plain @16:5..16:13]: "google.cloud.apigeeconnect.v1.Tether.Egress" [plain @16:15..16:58]
            "oauth" [plain @17:5..17:10]: mapping [block]
              "canonical_scopes" [plain @18:7..18:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @18:25..20:1]
    "#);
}
