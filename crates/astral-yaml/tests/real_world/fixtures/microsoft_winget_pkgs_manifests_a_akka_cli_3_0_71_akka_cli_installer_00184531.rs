#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-akka-cli-3-0-71-akka-cli-installer-00184531"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @1:1..1:18]: "Akka.Cli" [plain @1:20..1:28]
      "PackageVersion" [plain @2:1..2:15]: "3.0.71" [plain @2:17..2:23]
      "InstallerType" [plain @3:1..3:14]: "zip" [plain @3:16..3:19]
      "NestedInstallerType" [plain @4:1..4:20]: "portable" [plain @4:22..4:30]
      "NestedInstallerFiles" [plain @5:1..5:21]: sequence [block]
        - mapping [block]
          "RelativeFilePath" [plain @6:3..6:19]: "akka.exe" [plain @6:21..6:29]
          "PortableCommandAlias" [plain @7:3..7:23]: "akka" [plain @7:25..7:29]
      "Installers" [plain @8:1..8:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @9:3..9:15]: "x64" [plain @9:17..9:20]
          "InstallerUrl" [plain @10:3..10:15]: "https://downloads.akka.io/3.0.71/akka_windows_amd64_3.0.71.zip" [plain @10:17..10:79]
          "InstallerSha256" [plain @11:3..11:18]: "DACB7338C9EFB3924037DB11BECB00CEBD9555C1026BC253BEAE8D296DF32518" [plain @11:20..11:84]
      "ManifestType" [plain @12:1..12:13]: "installer" [plain @12:15..12:24]
      "ManifestVersion" [plain @13:1..13:16]: "1.6.0" [plain @13:18..13:23]
    "#);
}
