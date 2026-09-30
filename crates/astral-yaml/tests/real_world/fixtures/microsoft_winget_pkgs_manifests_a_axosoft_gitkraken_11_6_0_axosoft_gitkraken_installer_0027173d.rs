#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-axosoft-gitkraken-11-6-0-axosoft-gitkraken-installer-0027173d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Axosoft.GitKraken" [plain @4:20..4:37]
      "PackageVersion" [plain @5:1..5:15]: "11.6.0" [plain @5:17..5:23]
      "InstallerType" [plain @6:1..6:14]: "exe" [plain @6:16..6:19]
      "Scope" [plain @7:1..7:6]: "user" [plain @7:8..7:12]
      "InstallModes" [plain @8:1..8:13]: sequence [block]
        - "interactive" [plain @9:3..9:14]
        - "silent" [plain @10:3..10:9]
      "InstallerSwitches" [plain @11:1..11:18]: mapping [block]
        "Silent" [plain @12:3..12:9]: "--silent" [plain @12:11..12:19]
        "SilentWithProgress" [plain @13:3..13:21]: "--silent" [plain @13:23..13:31]
      "UpgradeBehavior" [plain @14:1..14:16]: "install" [plain @14:18..14:25]
      "Protocols" [plain @15:1..15:10]: sequence [block]
        - "gitkraken" [plain @16:3..16:12]
      "ProductCode" [plain @17:1..17:12]: "gitkraken" [plain @17:14..17:23]
      "ReleaseDate" [plain @18:1..18:12]: "2025-11-11" [plain @18:14..18:24]
      "Installers" [plain @19:1..19:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @20:3..20:15]: "x64" [plain @20:17..20:20]
          "InstallerUrl" [plain @21:3..21:15]: "https://api.gitkraken.dev/releases/production/windows/x64/11.6.0/GitKrakenSetup.exe" [plain @21:17..21:100]
          "InstallerSha256" [plain @22:3..22:18]: "11EAB08999B722C69AA318AEAE13F0CD7D2838B6DDC59D389891081B49215739" [plain @22:20..22:84]
      "ManifestType" [plain @23:1..23:13]: "installer" [plain @23:15..23:24]
      "ManifestVersion" [plain @24:1..24:16]: "1.10.0" [plain @24:18..24:24]
    "#);
}
