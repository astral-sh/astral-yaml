#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-azul-zulu-8-jdk-8-72-0-17-azul-zulu-8-jdk-locale-en-us-00167e9b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Azul.Zulu.8.JDK" [plain @4:20..4:35]
      "PackageVersion" [plain @5:1..5:15]: "8.72.0.17" [plain @5:17..5:26]
      "PackageLocale" [plain @6:1..6:14]: "en-US" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Azul Systems, Inc." [plain @7:12..7:30]
      "PublisherUrl" [plain @8:1..8:13]: "https://www.azul.com/" [plain @8:15..8:36]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://www.azul.com/support/" [plain @9:22..9:51]
      "PrivacyUrl" [plain @10:1..10:11]: "https://www.azul.com/privacy-policy/" [plain @10:13..10:49]
      "Author" [plain @11:1..11:7]: "Azul Systems, Inc." [plain @11:9..11:27]
      "PackageName" [plain @12:1..12:12]: "Azul Zulu JDK 8" [plain @12:14..12:29]
      "PackageUrl" [plain @13:1..13:11]: "https://www.azul.com/downloads/?package=jdk" [plain @13:13..13:56]
      "License" [plain @14:1..14:8]: "GPL 2 with Classpath Exception" [plain @14:10..14:40]
      "LicenseUrl" [plain @15:1..15:11]: "https://docs.azul.com/core/tpl" [plain @15:13..15:43]
      "Copyright" [plain @16:1..16:10]: "Copyright (c) 2013-2022, Azul Systems, Inc." [plain @16:12..16:55]
      "ShortDescription" [plain @17:1..17:17]: "Zulu 8 is a build of OpenJDK 8" [plain @17:19..17:49]
      "Moniker" [plain @18:1..18:8]: "zulu-jdk-8" [plain @18:10..18:20]
      "Tags" [plain @19:1..19:5]: sequence [block]
        - "java" [plain @20:3..20:7]
        - "jdk" [plain @21:3..21:6]
        - "openjdk" [plain @22:3..22:10]
        - "zulu" [plain @23:3..23:7]
      "ManifestType" [plain @24:1..24:13]: "defaultLocale" [plain @24:15..24:28]
      "ManifestVersion" [plain @25:1..25:16]: "1.5.0" [plain @25:18..25:23]
    "#);
}
