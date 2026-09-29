#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-authpass-authpass-1-9-1-1813-authpass-authpass-installer-0016c7b1"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Authpass.Authpass" [plain @4:20..4:37]
      "PackageVersion" [plain @5:1..5:15]: "1.9.1_1813" [plain @5:17..5:27]
      "InstallerLocale" [plain @6:1..6:16]: "en-US" [plain @6:18..6:23]
      "Platform" [plain @7:1..7:9]: sequence [block]
        - "Windows.Desktop" [plain @8:3..8:18]
      "MinimumOSVersion" [plain @9:1..9:17]: "10.0.0.0" [plain @9:19..9:27]
      "InstallerType" [plain @10:1..10:14]: "inno" [plain @10:16..10:20]
      "Scope" [plain @11:1..11:6]: "machine" [plain @11:8..11:15]
      "InstallModes" [plain @12:1..12:13]: sequence [block]
        - "interactive" [plain @13:3..13:14]
        - "silent" [plain @14:3..14:9]
        - "silentWithProgress" [plain @15:3..15:21]
      "UpgradeBehavior" [plain @16:1..16:16]: "install" [plain @16:18..16:25]
      "Installers" [plain @17:1..17:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @18:3..18:15]: "x64" [plain @18:17..18:20]
          "InstallerUrl" [plain @19:3..19:15]: "https://github.com/authpass/authpass/releases/download/v1.9.1/AuthPass-setup-1.9.1_1813.exe" [plain @19:17..19:108]
          "InstallerSha256" [plain @20:3..20:18]: "CC4A622F20D931D3EBBB82C3475DF51E1286F98B4351B217A01FB3CEF2DA4759" [plain @20:20..20:84]
      "ManifestType" [plain @21:1..21:13]: "installer" [plain @21:15..21:24]
      "ManifestVersion" [plain @22:1..22:16]: "1.9.0" [plain @22:18..22:23]
    "#);
}
