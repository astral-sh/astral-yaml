#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-9-9001-copyparty-1-20-16-9001-copyparty-installer-002be10f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "9001.copyparty" [plain @4:20..4:34]
      "PackageVersion" [plain @5:1..5:15]: "1.20.16" [plain @5:17..5:24]
      "InstallerType" [plain @6:1..6:14]: "portable" [plain @6:16..6:24]
      "InstallModes" [plain @7:1..7:13]: sequence [block]
        - "interactive" [plain @8:3..8:14]
        - "silent" [plain @9:3..9:9]
      "UpgradeBehavior" [plain @10:1..10:16]: "install" [plain @10:18..10:25]
      "ReleaseDate" [plain @11:1..11:12]: "2026-05-26" [plain @11:14..11:24]
      "Installers" [plain @12:1..12:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @13:3..13:15]: "x64" [plain @13:17..13:20]
          "InstallerUrl" [plain @14:3..14:15]: "https://github.com/9001/copyparty/releases/download/v1.20.16/copyparty.exe" [plain @14:17..14:91]
          "InstallerSha256" [plain @15:3..15:18]: "32723DF0B74E6BADA50921234127C2609934831CE835D51380F14CC3B6258FB4" [plain @15:20..15:84]
      "ManifestType" [plain @16:1..16:13]: "installer" [plain @16:15..16:24]
      "ManifestVersion" [plain @17:1..17:16]: "1.12.0" [plain @17:18..17:24]
    "#);
}
