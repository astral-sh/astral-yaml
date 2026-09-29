#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-apple-safari-5-34-57-2-apple-safari-installer-001e681e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @3:1..3:18]: "Apple.Safari" [plain @3:20..3:32]
      "PackageVersion" [plain @4:1..4:15]: "5.34.57.2" [plain @4:17..4:26]
      "ReleaseDate" [plain @5:1..5:12]: "2012-05-09" [plain @5:14..5:24]
      "InstallerType" [plain @6:1..6:14]: "exe" [plain @6:16..6:19]
      "InstallerSwitches" [plain @7:1..7:18]: mapping [block]
        "Silent" [plain @8:3..8:9]: "/qn" [plain @8:11..8:14]
        "SilentWithProgress" [plain @9:3..9:21]: "/qb" [plain @9:23..9:26]
      "Installers" [plain @10:1..10:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @11:3..11:15]: "x86" [plain @11:17..11:20]
          "InstallerUrl" [plain @12:3..12:15]: "http://appldnld.apple.com/Safari5/041-5487.20120509.INU8B/SafariSetup.exe" [plain @12:17..12:90]
          "InstallerSha256" [plain @13:3..13:18]: "155BB96BC972F227C563E2C3DB12EA00D212AF6C1FE882E0E944D6B5E1AAA3EE" [plain @13:20..13:84]
      "ManifestType" [plain @14:1..14:13]: "installer" [plain @14:15..14:24]
      "ManifestVersion" [plain @15:1..15:16]: "1.12.0" [plain @15:18..15:24]
    "#);
}
