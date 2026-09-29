#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-backend-provisioning-stacks-catalog-emulator-aws-096a3743"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "components" [plain @7:1..7:11]: mapping [block]
        "emulator" [plain @8:3..8:11]: mapping [block]
          "aws" [plain @9:5..9:8]: mapping [block]
            "driver" [plain @10:7..10:13]: "floci/aws" [plain @10:15..10:24]
            "region" [plain @11:7..11:13]: "us-east-1" [plain @11:15..11:24]
            "container" [plain @12:7..12:16]: mapping [block]
              "image" [plain @17:9..17:14]: "mirror.gcr.io/floci/floci@sha256:c88ec20bf221630dd195d38a14eeb0ac52ddfa72c37ebb3c8aa17f63ae27c5f2" [plain @17:16..17:113]
    "#);
}
