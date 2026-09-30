#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-alibaba-yuque-4-0-4-alibaba-yuque-locale-zh-cn-003b1aa1"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Alibaba.Yuque" [plain @4:20..4:33]
      "PackageVersion" [plain @5:1..5:15]: "4.0.4" [plain @5:17..5:22]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Yuque" [plain @7:12..7:17]
      "PublisherUrl" [plain @8:1..8:13]: "https://www.yuque.com" [plain @8:15..8:36]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://www.yuque.com/about/help" [plain @9:22..9:54]
      "PrivacyUrl" [plain @10:1..10:11]: "https://render.alipay.com/p/yuyan/180020010001196791/preview.html?agreementId=AG01000120" [plain @10:13..10:101]
      "Author" [plain @11:1..11:7]: "支付宝（杭州）信息技术有限公司" [plain @11:9..11:24]
      "PackageName" [plain @12:1..12:12]: "语雀" [plain @12:14..12:16]
      "PackageUrl" [plain @13:1..13:11]: "https://www.yuque.com/download" [plain @13:13..13:43]
      "License" [plain @14:1..14:8]: "专有软件" [plain @14:10..14:14]
      "LicenseUrl" [plain @15:1..15:11]: "https://www.yuque.com/terms" [plain @15:13..15:40]
      "Copyright" [plain @16:1..16:10]: "© 语雀" [plain @16:12..16:16]
      "ShortDescription" [plain @17:1..17:17]: "新一代云端知识库" [plain @17:19..17:27]
      "Description" [plain @18:1..18:12]: "个人笔记与知识创作，团队协同与知识沉淀" [plain @18:14..18:33]
      "Tags" [plain @19:1..19:5]: sequence [block]
        - "markdown" [plain @20:3..20:11]
        - "共享" [plain @21:3..21:5]
        - "协作" [plain @22:3..22:5]
        - "协同" [plain @23:3..23:5]
        - "博客" [plain @24:3..24:5]
        - "团队" [plain @25:3..25:5]
        - "图表" [plain @26:3..26:5]
        - "幻灯片" [plain @27:3..27:6]
        - "文档" [plain @28:3..28:5]
        - "演示" [plain @29:3..29:5]
        - "知识" [plain @30:3..30:5]
        - "知识库" [plain @31:3..31:6]
        - "笔记" [plain @32:3..32:5]
        - "编辑器" [plain @33:3..33:6]
        - "表格" [plain @34:3..34:5]
      "ReleaseNotes" [plain @35:1..35:13]: literal [@35:15..55:1]
        "- [🔥新增] 语雀 AI 功能正式上线，助力文档阅读、创作，随时待命\n"
        "- [🔥新增] 数据表新增图表视图\n"
        "- [优化] 代码块增加代码运行功能\n"
        "- [优化] 性能优化\n"
        "- [优化] 窗口拖拽能力优化\n"
        "- [优化] 开始 Tab 增加快捷跳转能力\n"
        "- [优化] 语雀空间用户搜索优化\n"
        "- [优化] 表格单元格样式设置逻辑优化\n"
        "- [优化] 目录中分组/链接类型节点的交互优化\n"
        "- [优化] 数据表性能优化\n"
        "- [优化] 文档支持解析协作链接\n"
        "- [修复] macOS15 系统版本导致的重复登录问题\n"
        "- [修复] 知识库页面报错问题\n"
        "- [修复] 文档内表格拖拽宽度报错的问题\n"
        "- [修复] 数据表的报错问题\n"
        "- [修复] 图册不支持 heic 的问题\n"
        "- [修复] 表格 INDIRECT 公式特定场景下的计算逻辑\n"
        "- [修复] 数据表图表视图筛选\n"
        "- [修复] 数据表多选和人员类型连续编辑"
      "ReleaseNotesUrl" [plain @55:1..55:16]: "https://www.yuque.com/yuque/yuque-desktop/changelog-4-0-x" [plain @55:18..55:75]
      "PurchaseUrl" [plain @56:1..56:12]: "https://www.yuque.com/about/price" [plain @56:14..56:47]
      "Documentations" [plain @57:1..57:15]: sequence [block]
        - mapping [block]
          "DocumentLabel" [plain @58:3..58:16]: "常见问题" [plain @58:18..58:22]
          "DocumentUrl" [plain @59:3..59:14]: "https://www.yuque.com/yuque/yuque-desktop/desktop-faq" [plain @59:16..59:69]
      "ManifestType" [plain @60:1..60:13]: "locale" [plain @60:15..60:21]
      "ManifestVersion" [plain @61:1..61:16]: "1.9.0" [plain @61:18..61:23]
    "#);
}
