#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-amazon-awscli-2-17-37-amazon-awscli-locale-en-us-003ccc9d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Amazon.AWSCLI" [plain @4:20..4:33]
      "PackageVersion" [plain @5:1..5:15]: "2.17.37" [plain @5:17..5:24]
      "PackageLocale" [plain @6:1..6:14]: "en-US" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Amazon Web Services" [plain @7:12..7:31]
      "PublisherUrl" [plain @8:1..8:13]: "https://github.com/aws/aws-cli" [plain @8:15..8:45]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://github.com/aws/aws-cli/issues" [plain @9:22..9:59]
      "Author" [plain @10:1..10:7]: "Amazon Web Services" [plain @10:9..10:28]
      "PackageName" [plain @11:1..11:12]: "AWS Command Line Interface" [plain @11:14..11:40]
      "PackageUrl" [plain @12:1..12:11]: "https://aws.amazon.com/cli" [plain @12:13..12:39]
      "License" [plain @13:1..13:8]: "Apache 2.0 license" [plain @13:10..13:28]
      "LicenseUrl" [plain @14:1..14:11]: "https://github.com/aws/aws-cli/blob/develop/LICENSE.txt" [plain @14:13..14:68]
      "Copyright" [plain @15:1..15:10]: "Copyright 2012-2020 Amazon.com, Inc." [plain @15:12..15:48]
      "CopyrightUrl" [plain @16:1..16:13]: "https://github.com/aws/aws-cli/blob/develop/LICENSE.txt" [plain @16:15..16:70]
      "ShortDescription" [plain @17:1..17:17]: "Universal Command Line Interface for Amazon Web Services" [plain @17:19..17:75]
      "Description" [plain @18:1..18:12]: literal [@18:14..21:1]
        "The AWS CLI is an open source tool built on top of the AWS SDK for Python (Boto) that provides commands for interacting with AWS services.\n"
        "With minimal configuration, you can start using all of the functionality provided by the AWS Management Console from your favorite terminal program."
      "Tags" [plain @21:1..21:5]: sequence [block]
        - "aws" [plain @22:3..22:6]
        - "awscli" [plain @23:3..23:9]
        - "cli" [plain @24:3..24:6]
        - "cloud" [plain @25:3..25:8]
        - "foss" [plain @26:3..26:7]
        - "open-source" [plain @27:3..27:14]
        - "s3" [plain @28:3..28:5]
        - "services" [plain @29:3..29:11]
        - "web" [plain @30:3..30:6]
      "ManifestType" [plain @31:1..31:13]: "defaultLocale" [plain @31:15..31:28]
      "ManifestVersion" [plain @32:1..32:16]: "1.6.0" [plain @32:18..32:23]
    "#);
}
