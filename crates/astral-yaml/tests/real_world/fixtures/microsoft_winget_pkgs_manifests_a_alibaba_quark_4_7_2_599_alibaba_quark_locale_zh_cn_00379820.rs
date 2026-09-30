#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-alibaba-quark-4-7-2-599-alibaba-quark-locale-zh-cn-00379820"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Alibaba.Quark" [plain @4:20..4:33]
      "PackageVersion" [plain @5:1..5:15]: "4.7.2.599" [plain @5:17..5:26]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Quark" [plain @7:12..7:17]
      "PublisherUrl" [plain @8:1..8:13]: "https://www.quark.cn/" [plain @8:15..8:36]
      "PrivacyUrl" [plain @9:1..9:11]: "https://terms.alicdn.com/legal-agreement/terms/c_end_product_protocol/20230727184418287/20230727184418287.html" [plain @9:13..9:123]
      "Author" [plain @10:1..10:7]: "广州市动悦信息技术有限公司" [plain @10:9..10:22]
      "PackageName" [plain @11:1..11:12]: "夸克" [plain @11:14..11:16]
      "PackageUrl" [plain @12:1..12:11]: "https://www.quark.cn/" [plain @12:13..12:34]
      "License" [plain @13:1..13:8]: "免费软件" [plain @13:10..13:14]
      "LicenseUrl" [plain @14:1..14:11]: "https://terms.alicdn.com/legal-agreement/terms/c_end_product_protocol/20230727183841535/20230727183841535.html" [plain @14:13..14:123]
      "Copyright" [plain @15:1..15:10]: "版权所有 2025 The Quark Authors。保留所有权利。" [plain @15:12..15:47]
      "ShortDescription" [plain @16:1..16:17]: "夸克是一款集搜索、网盘、工具等功能于一体的浏览器，为用户提供极速、智能、高效的体验。" [plain @16:19..16:61]
      "Tags" [plain @17:1..17:5]: sequence [block]
        - "chromium" [plain @18:3..18:11]
        - "互联网" [plain @19:3..19:6]
        - "浏览器" [plain @20:3..20:6]
        - "网页" [plain @21:3..21:5]
      "ManifestType" [plain @22:1..22:13]: "defaultLocale" [plain @22:15..22:28]
      "ManifestVersion" [plain @23:1..23:16]: "1.10.0" [plain @23:18..23:24]
    "#);
}
