#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-a2-ai-rv-0-14-0-a2-ai-rv-installer-00090f5c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "A2-Ai.rv" [plain @4:20..4:28]
      "PackageVersion" [plain @5:1..5:15]: "0.14.0" [plain @5:17..5:23]
      "InstallerType" [plain @6:1..6:14]: "zip" [plain @6:16..6:19]
      "NestedInstallerType" [plain @7:1..7:20]: "portable" [plain @7:22..7:30]
      "NestedInstallerFiles" [plain @8:1..8:21]: sequence [block]
        - mapping [block]
          "RelativeFilePath" [plain @9:3..9:19]: "rv.exe" [plain @9:21..9:27]
      "Commands" [plain @10:1..10:9]: sequence [block]
        - "rv" [plain @11:3..11:5]
      "Dependencies" [plain @12:1..12:13]: mapping [block]
        "PackageDependencies" [plain @13:3..13:22]: sequence [block]
          - mapping [block]
            "PackageIdentifier" [plain @14:5..14:22]: "Microsoft.VCRedist.2015+.x64" [plain @14:24..14:52]
      "ReleaseDate" [plain @15:1..15:12]: "2025-10-13" [plain @15:14..15:24]
      "Installers" [plain @16:1..16:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @17:3..17:15]: "x64" [plain @17:17..17:20]
          "InstallerUrl" [plain @18:3..18:15]: "https://github.com/A2-ai/rv/releases/download/v0.14.0/rv-v0.14.0-x86_64-pc-windows-msvc.zip" [plain @18:17..18:108]
          "InstallerSha256" [plain @19:3..19:18]: "2D26465A2EBD91018B099904087628F223680055B8F7A8BE196A2D8E2CDDBCB7" [plain @19:20..19:84]
      "ManifestType" [plain @20:1..20:13]: "installer" [plain @20:15..20:24]
      "ManifestVersion" [plain @21:1..21:16]: "1.10.0" [plain @21:18..21:24]
    "#);
}
