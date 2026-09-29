#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-altova-mobiletogetherdesigner-10-1-0-1-altova-mobiletogetherdesigner-locale-zh-cn-00010a39"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Altova.MobileTogetherDesigner" [plain @4:20..4:49]
      "PackageVersion" [plain @5:1..5:15]: "10.1.0.1" [plain @5:17..5:25]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "License" [plain @7:1..7:8]: "专有软件" [plain @7:10..7:14]
      "ShortDescription" [plain @8:1..8:17]: "跨平台移动开发环境" [plain @8:19..8:28]
      "Description" [plain @9:1..9:12]: "MobileTogether Designer 是一个免费的移动开发环境，无需移动开发经验也可以通过强大的可视化和功能性编程范例创建高级移动应用。要创建自定义的无代码数据库应用，请在 MobileTogether Designer 中使用 RecordsManager。" [plain @9:14..9:149]
      "Tags" [plain @10:1..10:5]: sequence [block]
        - "应用" [plain @11:3..11:5]
        - "应用程序" [plain @12:3..12:7]
        - "开发" [plain @13:3..13:5]
        - "手机应用" [plain @14:3..14:7]
      "Documentations" [plain @15:1..15:15]: sequence [block]
        - mapping [block]
          "DocumentLabel" [plain @16:3..16:16]: "文档" [plain @16:18..16:20]
          "DocumentUrl" [plain @17:3..17:14]: "https://www.altova.com/documentation" [plain @17:16..17:52]
        - mapping [block]
          "DocumentLabel" [plain @18:3..18:16]: "培训和教程" [plain @18:18..18:23]
          "DocumentUrl" [plain @19:3..19:14]: "https://www.altova.com/training-tutorials" [plain @19:16..19:57]
      "ManifestType" [plain @20:1..20:13]: "locale" [plain @20:15..20:21]
      "ManifestVersion" [plain @21:1..21:16]: "1.10.0" [plain @21:18..21:24]
    "#);
}
