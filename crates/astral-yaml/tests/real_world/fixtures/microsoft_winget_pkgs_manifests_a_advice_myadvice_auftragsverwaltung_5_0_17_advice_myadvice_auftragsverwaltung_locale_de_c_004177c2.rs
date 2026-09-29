#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-advice-myadvice-auftragsverwaltung-5-0-17-advice-myadvice-auftragsverwaltung-locale-de-c-004177c2"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Advice.myADVICE.Auftragsverwaltung" [plain @4:20..4:54]
      "PackageVersion" [plain @5:1..5:15]: "5.0.17" [plain @5:17..5:23]
      "PackageLocale" [plain @6:1..6:14]: "de-CH" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "ADVICE Informatik AG" [plain @7:12..7:32]
      "PackageName" [plain @8:1..8:12]: "myADVICE.Auftragsverwaltung" [plain @8:14..8:41]
      "License" [plain @9:1..9:8]: "Proprietary" [plain @9:10..9:21]
      "Copyright" [plain @10:1..10:10]: "ADVICE Informatik AG, 2020 - Switzerland." [plain @10:12..10:53]
      "ShortDescription" [plain @11:1..11:17]: "myADVICE.Auftragsverwaltung ist ein Kunden- und Rechnungsverwaltung (Offerte, Auftragsbestätigung, Rechnung, Serienrechnung) Applikation für kleine Unternehmen." [plain @11:19..11:179]
      "ManifestType" [plain @12:1..12:13]: "defaultLocale" [plain @12:15..12:28]
      "ManifestVersion" [plain @13:1..13:16]: "1.4.0" [plain @13:18..13:23]
    "#);
}
