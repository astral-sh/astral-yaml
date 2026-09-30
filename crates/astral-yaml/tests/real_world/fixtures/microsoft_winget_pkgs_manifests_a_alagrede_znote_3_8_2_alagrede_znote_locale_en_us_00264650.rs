#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-alagrede-znote-3-8-2-alagrede-znote-locale-en-us-00264650"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "alagrede.znote" [plain @4:20..4:34]
      "PackageVersion" [plain @5:1..5:15]: "3.8.2" [plain @5:17..5:22]
      "PackageLocale" [plain @6:1..6:14]: "en-US" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "alagrede" [plain @7:12..7:20]
      "PublisherUrl" [plain @8:1..8:13]: "https://github.com/alagrede" [plain @8:15..8:42]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://github.com/alagrede/znote-app/issues" [plain @9:22..9:66]
      "PackageName" [plain @10:1..10:12]: "znote" [plain @10:14..10:19]
      "PackageUrl" [plain @11:1..11:11]: "https://github.com/alagrede/znote-app" [plain @11:13..11:50]
      "License" [plain @12:1..12:8]: "Freeware" [plain @12:10..12:18]
      "ShortDescription" [plain @13:1..13:17]: "Markdown-based note-taking app for developers" [plain @13:19..13:64]
      "Tags" [plain @14:1..14:5]: sequence [block]
        - "dev" [plain @15:3..15:6]
        - "editor" [plain @16:3..16:9]
        - "electron" [plain @17:3..17:11]
        - "markdown" [plain @18:3..18:11]
        - "react" [plain @19:3..19:8]
        - "znote" [plain @20:3..20:8]
      "ReleaseNotes" [plain @21:1..21:13]: literal [@21:15..25:1]
        "🐛 Fixes\n"
        "- Improved meeting transcription reliability and overall transcription quality\n"
        "- Fixed several issues related to meeting recording and transcript generation"
      "ReleaseNotesUrl" [plain @25:1..25:16]: "https://github.com/alagrede/znote-app/releases/tag/v3.8.2" [plain @25:18..25:75]
      "Documentations" [plain @26:1..26:15]: sequence [block]
        - mapping [block]
          "DocumentLabel" [plain @27:3..27:16]: "FAQ" [plain @27:18..27:21]
          "DocumentUrl" [plain @28:3..28:14]: "https://znote.io/faq.html" [plain @28:16..28:41]
      "ManifestType" [plain @29:1..29:13]: "defaultLocale" [plain @29:15..29:28]
      "ManifestVersion" [plain @30:1..30:16]: "1.12.0" [plain @30:18..30:24]
    "#);
}
