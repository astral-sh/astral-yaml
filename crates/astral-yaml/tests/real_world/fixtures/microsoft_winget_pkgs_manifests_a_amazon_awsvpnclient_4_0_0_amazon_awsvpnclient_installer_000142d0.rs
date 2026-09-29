#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-amazon-awsvpnclient-4-0-0-amazon-awsvpnclient-installer-000142d0"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Amazon.AWSVPNClient" [plain @4:20..4:39]
      "PackageVersion" [plain @5:1..5:15]: "4.0.0" [plain @5:17..5:22]
      "InstallerLocale" [plain @6:1..6:16]: "en-US" [plain @6:18..6:23]
      "InstallerType" [plain @7:1..7:14]: "wix" [plain @7:16..7:19]
      "Scope" [plain @8:1..8:6]: "machine" [plain @8:8..8:15]
      "ProductCode" [plain @9:1..9:12]: "{597A89E3-C473-4917-B0E2-6968A42E538B}" [single-quoted @9:14..9:54]
      "AppsAndFeaturesEntries" [plain @10:1..10:23]: sequence [block]
        - mapping [block]
          "UpgradeCode" [plain @11:3..11:14]: "{8D1975AA-4987-4668-ACD2-66EC556E5608}" [single-quoted @11:16..11:56]
      "Installers" [plain @12:1..12:11]: sequence [block]
        - mapping [block]
          "Architecture" [plain @13:3..13:15]: "x64" [plain @13:17..13:20]
          "InstallerUrl" [plain @14:3..14:15]: "https://d20adtppz83p9s.cloudfront.net/WPF/4.0.0/AWS_VPN_Client.msi" [plain @14:17..14:83]
          "InstallerSha256" [plain @15:3..15:18]: "6532F911385EC8FAC1494D0847C8F90A999B3BD7380844E2EA4318E9DB4A2EBC" [plain @15:20..15:84]
      "ManifestType" [plain @16:1..16:13]: "installer" [plain @16:15..16:24]
      "ManifestVersion" [plain @17:1..17:16]: "1.6.0" [plain @17:18..17:23]
    "#);
}
