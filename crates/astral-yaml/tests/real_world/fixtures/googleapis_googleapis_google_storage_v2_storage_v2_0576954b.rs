#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-storage-v2-storage-v2-0576954b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "storage.googleapis.com" [plain @3:7..3:29]
      "title" [plain @4:1..4:6]: "Cloud Storage API" [plain @4:8..4:25]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.longrunning.Operations" [plain @7:9..7:38]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.storage.v2.Storage" [plain @8:9..8:34]
      "types" [plain @10:1..10:6]: sequence [block]
        - mapping [block]
          "name" [plain @11:3..11:7]: "google.storage.v2.Bucket" [plain @11:9..11:33]
      "enums" [plain @13:1..13:6]: sequence [block]
        - mapping [block]
          "name" [plain @14:3..14:7]: "google.storage.v2.ServiceConstants.Values" [plain @14:9..14:50]
      "documentation" [plain @16:1..16:14]: mapping [block]
        "summary" [plain @17:3..17:10]: "Lets you store and retrieve potentially-large, immutable data objects." [single-quoted @17:12..17:84]
        "overview" [plain @18:3..18:11]: literal [@18:13..27:1]
          "The Google Cloud Storage API allows applications to read and write data\n"
          "through the abstractions of buckets and objects, which are similar to\n"
          "directories and files except that buckets cannot contain other buckets,\n"
          "and directory-level operations (like directory rename) are not supported.\n"
          "Buckets share a single global namespace, and each bucket belongs to a\n"
          "specific project that has an associated owner that pays for the data\n"
          "stored in the bucket. This API is accessed using standard gRPC requests."
      "authentication" [plain @27:1..27:15]: mapping [block]
        "rules" [plain @28:3..28:8]: sequence [block]
          - mapping [block]
            "selector" [plain @29:5..29:13]: "google.longrunning.Operations.*" [single-quoted @29:15..29:48]
            "oauth" [plain @30:5..30:10]: mapping [block]
              "canonical_scopes" [plain @31:7..31:23]: literal [@31:25..37:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only,\n"
                "https://www.googleapis.com/auth/devstorage.full_control,\n"
                "https://www.googleapis.com/auth/devstorage.read_only,\n"
                "https://www.googleapis.com/auth/devstorage.read_write"
          - mapping [block]
            "selector" [plain @37:5..37:13]: "google.storage.v2.Storage.*" [single-quoted @37:15..37:44]
            "oauth" [plain @38:5..38:10]: mapping [block]
              "canonical_scopes" [plain @39:7..39:23]: literal [@39:25..45:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only,\n"
                "https://www.googleapis.com/auth/devstorage.full_control,\n"
                "https://www.googleapis.com/auth/devstorage.read_only,\n"
                "https://www.googleapis.com/auth/devstorage.read_write"
          - mapping [block]
            "selector" [plain @45:5..45:13]: "google.storage.v2.Storage.BidiWriteObject" [plain @45:15..45:56]
            "oauth" [plain @46:5..46:10]: mapping [block]
              "canonical_scopes" [plain @47:7..47:23]: literal [@47:25..52:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/devstorage.full_control,\n"
                "https://www.googleapis.com/auth/devstorage.read_write"
      "publishing" [plain @52:1..52:11]: mapping [block]
        "documentation_uri" [plain @53:3..53:20]: "https://cloud.google.com/storage/docs/apis" [plain @53:22..53:64]
        "library_settings" [plain @54:3..54:19]: sequence [block]
          - mapping [block]
            "version" [plain @55:5..55:12]: "google.storage.v2" [plain @55:14..55:31]
            "launch_stage" [plain @56:5..56:17]: "GA" [plain @56:19..56:21]
            "node_settings" [plain @57:5..57:18]: mapping [block]
              "common" [plain @58:7..58:13]: mapping [block]
                "selective_gapic_generation" [plain @59:9..59:35]: mapping [block]
                  "methods" [plain @60:11..60:18]: sequence [block]
                    - "google.storage.v2.Storage.GetBucket" [plain @61:13..61:48]
                    - "google.storage.v2.Storage.ListBuckets" [plain @62:13..62:50]
                    - "google.storage.v2.Storage.DeleteBucket" [plain @63:13..63:51]
                    - "google.storage.v2.Storage.CreateBucket" [plain @64:13..64:51]
                    - "google.storage.v2.Storage.LockBucketRetentionPolicy" [plain @65:13..65:64]
                    - "google.storage.v2.Storage.UpdateBucket" [plain @66:13..66:51]
    "#);
}
