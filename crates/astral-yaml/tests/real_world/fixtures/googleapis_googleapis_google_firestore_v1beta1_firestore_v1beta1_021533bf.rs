#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-firestore-v1beta1-firestore-v1beta1-021533bf"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "firestore.googleapis.com" [plain @3:7..3:31]
      "title" [plain @4:1..4:6]: "Cloud Firestore API" [plain @4:8..4:27]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.firestore.v1beta1.Firestore" [plain @7:9..7:43]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: literal [@10:12..13:1]
          "Accesses the NoSQL document database built for automatic scaling, high\n"
          "performance, and ease of application development."
        "rules" [plain @13:3..13:8]: sequence [block]
          - mapping [block]
            "selector" [plain @14:5..14:13]: "google.cloud.location.Locations.GetLocation" [plain @14:15..14:58]
            "description" [plain @15:5..15:16]: "Gets information about a location." [plain @15:18..15:52]
          - mapping [block]
            "selector" [plain @17:5..17:13]: "google.cloud.location.Locations.ListLocations" [plain @17:15..17:60]
            "description" [plain @18:5..18:16]: "Lists information about the supported locations for this service." [plain @18:18..18:83]
      "backend" [plain @20:1..20:8]: mapping [block]
        "rules" [plain @21:3..21:8]: sequence [block]
          - mapping [block]
            "selector" [plain @22:5..22:13]: "google.cloud.location.Locations.GetLocation" [plain @22:15..22:58]
            "deadline" [plain @23:5..23:13]: "295.0" [plain @23:15..23:20]
          - mapping [block]
            "selector" [plain @24:5..24:13]: "google.cloud.location.Locations.ListLocations" [plain @24:15..24:60]
            "deadline" [plain @25:5..25:13]: "295.0" [plain @25:15..25:20]
          - mapping [block]
            "selector" [plain @26:5..26:13]: "google.firestore.v1beta1.Firestore.*" [single-quoted @26:15..26:53]
            "deadline" [plain @27:5..27:13]: "295.0" [plain @27:15..27:20]
          - mapping [block]
            "selector" [plain @28:5..28:13]: "google.longrunning.Operations.*" [single-quoted @28:15..28:48]
            "deadline" [plain @29:5..29:13]: "295.0" [plain @29:15..29:20]
      "authentication" [plain @31:1..31:15]: mapping [block]
        "rules" [plain @32:3..32:8]: sequence [block]
          - mapping [block]
            "selector" [plain @33:5..33:13]: "google.cloud.location.Locations.GetLocation" [plain @33:15..33:58]
            "oauth" [plain @34:5..34:10]: mapping [block]
              "canonical_scopes" [plain @35:7..35:23]: literal [@35:25..38:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/datastore"
          - mapping [block]
            "selector" [plain @38:5..38:13]: "google.cloud.location.Locations.ListLocations" [plain @38:15..38:60]
            "oauth" [plain @39:5..39:10]: mapping [block]
              "canonical_scopes" [plain @40:7..40:23]: literal [@40:25..43:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/datastore"
          - mapping [block]
            "selector" [plain @43:5..43:13]: "google.firestore.v1beta1.Firestore.*" [single-quoted @43:15..43:53]
            "oauth" [plain @44:5..44:10]: mapping [block]
              "canonical_scopes" [plain @45:7..45:23]: literal [@45:25..48:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/datastore"
          - mapping [block]
            "selector" [plain @48:5..48:13]: "google.longrunning.Operations.*" [single-quoted @48:15..48:48]
            "oauth" [plain @49:5..49:10]: mapping [block]
              "canonical_scopes" [plain @50:7..50:23]: literal [@50:25..54:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/datastore"
      "publishing" [plain @54:1..54:11]: mapping [block]
        "library_settings" [plain @55:3..55:19]: sequence [block]
          - mapping [block]
            "version" [plain @56:5..56:12]: "google.firestore.v1beta1" [plain @56:14..56:38]
            "java_settings" [plain @57:5..57:18]: mapping [block]
              "library_package" [plain @58:7..58:22]: "com.google.cloud.firestore.v1beta1" [plain @58:24..58:58]
    "#);
}
