#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-3-360-360chrome-x-21-0-1040-0-360-360chrome-x-locale-zh-cn-0012625f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "360.360Chrome.X" [plain @4:20..4:35]
      "PackageVersion" [plain @5:1..5:15]: "21.0.1040.0" [plain @5:17..5:28]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "360安全中心" [plain @7:12..7:19]
      "PublisherUrl" [plain @8:1..8:13]: "https://browser.360.cn/" [plain @8:15..8:38]
      "PublisherSupportUrl" [plain @9:1..9:20]: "https://bbs.360.cn/forum-142-1.html" [plain @9:22..9:57]
      "PrivacyUrl" [plain @10:1..10:11]: "http://www.360.cn/privacy/v3/bps360jsllqWindowsb.html" [plain @10:13..10:66]
      "Author" [plain @11:1..11:7]: "北京奇虎科技有限公司" [plain @11:9..11:19]
      "PackageName" [plain @12:1..12:12]: "360 极速浏览器X" [plain @12:14..12:24]
      "PackageUrl" [plain @13:1..13:11]: "https://browser.360.cn/ee/" [plain @13:13..13:39]
      "License" [plain @14:1..14:8]: "免费软件" [plain @14:10..14:14]
      "LicenseUrl" [plain @15:1..15:11]: "http://www.360.cn/xukexieyi.html" [plain @15:13..15:45]
      "Copyright" [plain @16:1..16:10]: "(C) 360.cn All Rights Reserved." [plain @16:12..16:43]
      "CopyrightUrl" [plain @17:1..17:13]: "http://www.360.cn/about/banquanshengming.html" [plain @17:15..17:60]
      "ShortDescription" [plain @18:1..18:17]: "全新 Chromium 内核，64 位双核浏览器，性能更高，体验更佳" [plain @18:19..18:53]
      "Description" [plain @19:1..19:12]: "360 极速浏览器 X 是 360 推出的首款融合了最新 Chromium 内核的 64 位双核浏览器，较传统的 32 位浏览器，性能提升超 50%。在带给您更加畅快的底层浏览体验同时，我们也还为您提供了全新的极简风格操作界面、可以更换上千款高清壁纸的新标签页及众多精心优化过的浏览工具。" [plain @19:14..19:158]
      "Tags" [plain @21:1..21:5]: sequence [block]
        - "浏览器" [plain @22:3..22:6]
        - "Chromium" [plain @23:3..23:11]
        - "IE" [plain @24:3..24:5]
        - "Internet-Explorer" [plain @25:3..25:20]
        - "360" [double-quoted @26:3..26:8]
      "ReleaseNotes" [plain @28:1..28:13]: literal [@28:15..39:1]
        "1、浏览器一期用户需求-浏览器界面实现双屏及多屏浏览需求\n"
        "2、用户可实现默认浏览器管理设置\n"
        "3、修复用户反馈的部分税务网站智能切核状态异常问题\n"
        "4、优化冷启动调起浏览器速度\n"
        "5、修复用户反馈的部分扩展失效问题\n"
        "6、修复用户反馈的 Win11 系统状态下浏览器白边问题\n"
        "7、修复用户反馈的 GIF 动图复制粘贴后为静态图问题\n"
        "8、修复用户反馈的集中性崩溃问题\n"
        "9、用户反馈的部分网页登录失败问题\n"
        "10、其他已知问题修复"
      "ReleaseNotesUrl" [plain @39:1..39:16]: "https://bbs.360.cn/thread-16005307-1-1.html" [plain @39:18..39:61]
      "ManifestType" [plain @40:1..40:13]: "locale" [plain @40:15..40:21]
      "ManifestVersion" [plain @41:1..41:16]: "1.1.0" [plain @41:18..41:23]
    "#);
}
