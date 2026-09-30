#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-5-5rahim-seanime-2-4-2-5rahim-seanime-locale-en-us-000c1a25"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "5rahim.Seanime" [plain @4:20..4:34]
      "PackageVersion" [plain @5:1..5:15]: "2.4.2" [plain @5:17..5:22]
      "PackageLocale" [plain @6:1..6:14]: "en-US" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "seanime" [plain @7:12..7:19]
      "PublisherUrl" [plain @8:1..8:13]: "https://github.com/5rahim" [plain @8:15..8:40]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://github.com/5rahim/seanime/issues" [plain @9:22..9:62]
      "Author" [plain @10:1..10:7]: "5rahim" [plain @10:9..10:15]
      "PackageName" [plain @11:1..11:12]: "Seanime Desktop" [plain @11:14..11:29]
      "PackageUrl" [plain @12:1..12:11]: "https://github.com/5rahim/seanime" [plain @12:13..12:46]
      "License" [plain @13:1..13:8]: "GPL-3.0" [plain @13:10..13:17]
      "LicenseUrl" [plain @14:1..14:11]: "https://github.com/5rahim/seanime/blob/HEAD/LICENSE" [plain @14:13..14:64]
      "Copyright" [plain @15:1..15:10]: "Copyright (c) 5rahim" [plain @15:12..15:32]
      "CopyrightUrl" [plain @16:1..16:13]: "https://github.com/5rahim/seanime/blob/main/LICENSE" [plain @16:15..16:66]
      "ShortDescription" [plain @17:1..17:17]: "Open-source media server with a Web UI and Desktop app for anime and manga." [plain @17:19..17:94]
      "Description" [plain @18:1..18:12]: "Seanime is a media server with a web interface and desktop app for watching anime, reading manga, and managing your local library. Seanime provides a collection of features to enhance your watching and reading experience. Local library management, transcoding, Bittorrent or Debrid downloading and streaming, offline viewing, and more." [plain @18:14..18:349]
      "Moniker" [plain @19:1..19:8]: "seanime-desktop" [plain @19:10..19:25]
      "Tags" [plain @20:1..20:5]: sequence [block]
        - "anilist" [plain @21:3..21:10]
        - "anime" [plain @22:3..22:8]
        - "anime-downloader" [plain @23:3..23:19]
        - "anime-library" [plain @24:3..24:16]
        - "anime-streaming" [plain @25:3..25:18]
        - "bittorrent" [plain @26:3..26:13]
        - "debrid" [plain @27:3..27:9]
        - "go" [plain @28:3..28:5]
        - "manga" [plain @29:3..29:8]
        - "manga-downloader" [plain @30:3..30:19]
        - "media-server" [plain @31:3..31:15]
        - "myanimelist" [plain @32:3..32:14]
        - "react" [plain @33:3..33:8]
        - "scanner" [plain @34:3..34:10]
        - "self-hosted" [plain @35:3..35:14]
      "ReleaseNotes" [plain @36:1..36:13]: literal [@36:15..46:1]
        "What's new?\n"
        "- ⚡\u{fe0f} 'Include in library' will keep displaying shows when caught up\n"
        "- ⚡\u{fe0f} Settings: Open data directory button\n"
        "- 🦺 Desktop: Fixed authentication issue on macOS\n"
        "- ⚡\u{fe0f} Desktop: Force single instance\n"
        "- ⚡\u{fe0f} Desktop: Try to shut down server on force exit\n"
        "- ⚡\u{fe0f} Desktop: Disallow update from Web UI\n"
        "- 🦺 Desktop: Fixed 'toggle visibility'\n"
        "- 🦺 Desktop: Fixed 'server process terminated' issue"
      "ReleaseNotesUrl" [plain @46:1..46:16]: "https://github.com/5rahim/seanime/releases/tag/v2.4.2" [plain @46:18..46:71]
      "ManifestType" [plain @47:1..47:13]: "defaultLocale" [plain @47:15..47:28]
      "ManifestVersion" [plain @48:1..48:16]: "1.12.0" [plain @48:18..48:24]
    "#);
}
