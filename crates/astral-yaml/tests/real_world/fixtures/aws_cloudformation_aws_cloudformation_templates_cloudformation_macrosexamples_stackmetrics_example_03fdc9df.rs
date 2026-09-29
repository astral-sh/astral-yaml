#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-cloudformation-macrosexamples-stackmetrics-example-03fdc9df"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "Transform" [plain @1:1..1:10]: "StackMetrics" [plain @1:12..1:24]
      "Resources" [plain @3:1..3:10]: mapping [block]
        "Bucket1" [plain @4:3..4:10]: mapping [block]
          "Type" [plain @5:5..5:9]: "AWS::S3::Bucket" [plain @5:11..5:26]
          "Metadata" [plain @6:5..6:13]: mapping [block]
            "Comment" [plain @7:7..7:14]: "Suppressing typical rules for sample purposes only" [plain @7:16..7:66]
            "guard" [plain @8:7..8:12]: mapping [block]
              "SuppressedRules" [plain @9:9..9:24]: sequence [block]
                - "S3_BUCKET_LOGGING_ENABLED" [plain @10:13..10:38]
                - "S3_BUCKET_PUBLIC_READ_PROHIBITED" [plain @11:13..11:45]
                - "S3_BUCKET_PUBLIC_WRITE_PROHIBITED" [plain @12:13..12:46]
                - "S3_BUCKET_REPLICATION_ENABLED" [plain @13:13..13:42]
                - "S3_BUCKET_VERSIONING_ENABLED" [plain @14:13..14:41]
                - "S3_BUCKET_DEFAULT_LOCK_ENABLED" [plain @15:13..15:43]
                - "S3_BUCKET_SERVER_SIDE_ENCRYPTION_ENABLED" [plain @16:13..16:53]
    "#);
}
