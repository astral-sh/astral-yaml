#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-audacity-audacity-3-7-7-audacity-audacity-locale-en-us-0024f5f1"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Audacity.Audacity" [plain @4:20..4:37]
      "PackageVersion" [plain @5:1..5:15]: "3.7.7" [plain @5:17..5:22]
      "PackageLocale" [plain @6:1..6:14]: "en-US" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Audacity Team" [plain @7:12..7:25]
      "PublisherUrl" [plain @8:1..8:13]: "https://www.audacityteam.org/" [plain @8:15..8:44]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://support.audacityteam.org/" [plain @9:22..9:55]
      "PrivacyUrl" [plain @10:1..10:11]: "https://www.audacityteam.org/desktop-privacy-notice/" [plain @10:13..10:65]
      "Author" [plain @11:1..11:7]: "MuseCY SM Ltd." [plain @11:9..11:23]
      "PackageName" [plain @12:1..12:12]: "Audacity" [plain @12:14..12:22]
      "PackageUrl" [plain @13:1..13:11]: "https://www.audacityteam.org/" [plain @13:13..13:42]
      "License" [plain @14:1..14:8]: "GPL-3.0-or-later" [plain @14:10..14:26]
      "LicenseUrl" [plain @15:1..15:11]: "https://github.com/audacity/audacity/blob/HEAD/LICENSE.txt" [plain @15:13..15:71]
      "Copyright" [plain @16:1..16:10]: "Copyright © 1999-2025 Audacity Team." [plain @16:12..16:48]
      "ShortDescription" [plain @17:1..17:17]: "The world’s most popular free software for recording and editing audio." [plain @17:19..17:90]
      "Description" [plain @18:1..18:12]: literal [@18:14..27:1]
        "Audacity is the world’s most popular free software for recording and editing audio. So if you're producing music, a podcast, or just playing around with audio, Audacity is for you.\n"
        "\n"
        "Audacity is for anyone who wants to get creative with sound. It’s also the perfect tool for anyone who needs to quickly edit or export audio, for any reason.\n"
        "\n"
        "Cross-platform：Runs on all major operating systems – Windows, MacOS and Linux.\n"
        "Import, export, convert：Audacity supports all major audio formats, allowing you to convert WAV to MP3, FLAC, Ogg and much more.\n"
        "Plugin support：Elevate your productions with a vast selection of third-party plugins, including VST3, Nyquist and more.\n"
        "Deep audio analysis：Visualize frequencies in Audacity's Spectogram view or use scientific Vamp analyzers to make discoveries."
      "Moniker" [plain @27:1..27:8]: "audacity" [plain @27:10..27:18]
      "Tags" [plain @28:1..28:5]: sequence [block]
        - "audio" [plain @29:3..29:8]
        - "audio-editor" [plain @30:3..30:15]
        - "audioeditor" [plain @31:3..31:14]
        - "musegroup" [plain @32:3..32:12]
        - "musicstudio" [plain @33:3..33:14]
        - "music-studio" [plain @34:3..34:15]
        - "audiorecording" [plain @35:3..35:17]
        - "audio-recording" [plain @36:3..36:18]
        - "soundrecording" [plain @37:3..37:17]
        - "sound-recording" [plain @38:3..38:18]
        - "soundediting" [plain @39:3..39:15]
        - "xp8k0j757hhrdw" [plain @40:3..40:17]
      "ReleaseNotes" [plain @41:1..41:13]: literal [@41:15..50:1]
        "This is a hotfix release. It fixes broken waveform scrolling and selection for some users introduced in 3.7.6.\n"
        "The fix was provided by @iv-m and @DavidBailes, thank you!\n"
        "Other changes：\n"
        "• #9940 Added checksum to WavPack export (thanks @ajsand)\n"
        "New Contributors\n"
        "• @iv-m made their first contribution in https://github.com/audacity/audacity/pull/10011\n"
        "• @ajsand made their first contribution in https://github.com/audacity/audacity/pull/9940\n"
        "Full Changelog：https://github.com/audacity/audacity/compare/Audacity-3.7.6...Audacity-3.7.7"
      "ReleaseNotesUrl" [plain @50:1..50:16]: "https://github.com/audacity/audacity/releases/tag/Audacity-3.7.7" [plain @50:18..50:82]
      "Documentations" [plain @51:1..51:15]: sequence [block]
        - mapping [block]
          "DocumentLabel" [plain @52:3..52:16]: "FAQ" [plain @52:18..52:21]
          "DocumentUrl" [plain @53:3..53:14]: "https://www.audacityteam.org/faq/" [plain @53:16..53:49]
        - mapping [block]
          "DocumentLabel" [plain @54:3..54:16]: "Manual" [plain @54:18..54:24]
          "DocumentUrl" [plain @55:3..55:14]: "https://manual.audacityteam.org/" [plain @55:16..55:48]
        - mapping [block]
          "DocumentLabel" [plain @56:3..56:16]: "Wiki" [plain @56:18..56:22]
          "DocumentUrl" [plain @57:3..57:14]: "https://github.com/audacity/audacity/wiki" [plain @57:16..57:57]
      "ManifestType" [plain @58:1..58:13]: "defaultLocale" [plain @58:15..58:28]
      "ManifestVersion" [plain @59:1..59:16]: "1.10.0" [plain @59:18..59:24]
    "#);
}
