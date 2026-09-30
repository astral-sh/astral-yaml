#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-amazon-awscloudhsm-ksp-5-18-0-amazon-awscloudhsm-ksp-locale-zh-cn-0028533c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Amazon.AWSCloudHSM.KSP" [plain @4:20..4:42]
      "PackageVersion" [plain @5:1..5:15]: "5.18.0" [plain @5:17..5:23]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "PublisherUrl" [plain @7:1..7:13]: "https://aws.amazon.com/cn/" [plain @7:15..7:41]
      "PrivacyUrl" [plain @8:1..8:11]: "https://aws.amazon.com/cn/privacy/" [plain @8:13..8:47]
      "PackageUrl" [plain @9:1..9:11]: "https://docs.aws.amazon.com/zh_cn/cloudhsm/latest/userguide/latest-releases.html" [plain @9:13..9:93]
      "License" [plain @10:1..10:8]: "专有软件" [plain @10:10..10:14]
      "LicenseUrl" [plain @11:1..11:11]: "https://aws.amazon.com/cn/agreement/" [plain @11:13..11:49]
      "CopyrightUrl" [plain @12:1..12:13]: "https://aws.amazon.com/cn/agreement/" [plain @12:15..12:51]
      "ShortDescription" [plain @13:1..13:17]: "适用于 Windows 的 AWS CloudHSM 客户端包含 CNG 和 KSP 提供程序。" [plain @13:19..13:67]
      "ReleaseNotesUrl" [plain @14:1..14:16]: "https://docs.aws.amazon.com/zh_cn/cloudhsm/latest/userguide/client-history.html" [plain @14:18..14:97]
      "ManifestType" [plain @15:1..15:13]: "locale" [plain @15:15..15:21]
      "ManifestVersion" [plain @16:1..16:16]: "1.12.0" [plain @16:18..16:24]
    "#);
}
