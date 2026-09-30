#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-a2-ai-rv-0-21-0-a2-ai-rv-locale-zh-cn-00250534"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "A2-Ai.rv" [plain @4:20..4:28]
      "PackageVersion" [plain @5:1..5:15]: "0.21.0" [plain @5:17..5:23]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "ShortDescription" [plain @7:1..7:17]: "可重现、快速、声明式的 R 语言包管理和安装。" [plain @7:19..7:42]
      "ReleaseNotesUrl" [plain @8:1..8:16]: "https://github.com/A2-ai/rv/releases/tag/v0.21.0" [plain @8:18..8:66]
      "ManifestType" [plain @9:1..9:13]: "locale" [plain @9:15..9:21]
      "ManifestVersion" [plain @10:1..10:16]: "1.12.0" [plain @10:18..10:24]
    "#);
}
