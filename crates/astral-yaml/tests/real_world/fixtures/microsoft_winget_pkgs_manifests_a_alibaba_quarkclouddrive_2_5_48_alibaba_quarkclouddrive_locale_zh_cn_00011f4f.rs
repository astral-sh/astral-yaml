#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-alibaba-quarkclouddrive-2-5-48-alibaba-quarkclouddrive-locale-zh-cn-00011f4f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "Alibaba.QuarkCloudDrive" [plain @4:20..4:43]
      "PackageVersion" [plain @5:1..5:15]: "2.5.48" [plain @5:17..5:23]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "Publisher" [plain @7:1..7:10]: "Quark" [plain @7:12..7:17]
      "PublisherUrl" [plain @8:1..8:13]: "https://www.myquark.cn/" [plain @8:15..8:38]
      "PrivacyUrl" [plain @10:1..10:11]: "https://terms.alicdn.com/legal-agreement/terms/suit_bu1_uc/suit_bu1_uc202002251416_20839.html" [plain @10:13..10:106]
      "Author" [plain @11:1..11:7]: "广州市动景计算机科技有限公司" [plain @11:9..11:23]
      "PackageName" [plain @12:1..12:12]: "夸克网盘" [plain @12:14..12:18]
      "PackageUrl" [plain @13:1..13:11]: "https://pan.quark.cn/" [plain @13:13..13:34]
      "License" [plain @14:1..14:8]: "专有软件" [plain @14:10..14:14]
      "LicenseUrl" [plain @15:1..15:11]: "https://broccoli.uc.cn/apps/quark_icloud_pay/routes/service_help" [plain @15:13..15:77]
      "Copyright" [plain @16:1..16:10]: "Copyright © 2023 Quark" [plain @16:12..16:34]
      "ShortDescription" [plain @18:1..18:17]: "夸克网盘是夸克推出的一款云服务产品，功能包括云存储、高清看剧、文件在线解压、PDF 一键转换等。" [plain @18:19..18:67]
      "Tags" [plain @21:1..21:5]: sequence [block]
        - "上传" [plain @22:3..22:5]
        - "下载" [plain @23:3..23:5]
        - "云" [plain @24:3..24:4]
        - "云盘" [plain @25:3..25:5]
        - "共享" [plain @26:3..26:5]
        - "分享" [plain @27:3..27:5]
        - "同步" [plain @28:3..28:5]
        - "备份" [plain @29:3..29:5]
        - "文件" [plain @30:3..30:5]
        - "网盘" [plain @31:3..31:5]
      "ReleaseNotes" [plain @32:1..32:13]: literal [@32:15..37:1]
        "备份能力全新升级，支持三种备份模式，省电不打扰，智能又贴心。\n"
        "1. 新增低耗能备份，不抢占电脑资源，状态好时再备份，电脑内存占用过高、低电量时暂停备份。\n"
        "2. 新增定时备份，闲时备份不打扰，夜间、课间、午休、下班后任你设置\n"
        "如果遇到体验问题，可以通过夸克 App-个人中心-设置-客服咨询反馈，或者联系：020-66824888"
      "ManifestType" [plain @41:1..41:13]: "locale" [plain @41:15..41:21]
      "ManifestVersion" [plain @42:1..42:16]: "1.5.0" [plain @42:18..42:23]
    "#);
}
