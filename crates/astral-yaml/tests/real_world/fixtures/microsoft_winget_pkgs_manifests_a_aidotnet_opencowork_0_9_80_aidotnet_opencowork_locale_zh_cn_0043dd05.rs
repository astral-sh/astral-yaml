#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("microsoft-winget-pkgs-manifests-a-aidotnet-opencowork-0-9-80-aidotnet-opencowork-locale-zh-cn-0043dd05"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "PackageIdentifier" [plain @4:1..4:18]: "AIDotNet.OpenCowork" [plain @4:20..4:39]
      "PackageVersion" [plain @5:1..5:15]: "0.9.80" [plain @5:17..5:23]
      "PackageLocale" [plain @6:1..6:14]: "zh-CN" [plain @6:16..6:21]
      "ShortDescription" [plain @7:1..7:17]: "开源桌面多智能体协作平台，为 AI 智能体赋予本地工具、并行团队协作及无缝办公集成能力。" [plain @7:19..7:63]
      "Description" [plain @8:1..8:12]: literal [@8:14..15:1]
        "传统的 LLM 界面往往是“环境孤岛”。开发者通常需要花费 50% 的时间在聊天窗口和 IDE 之间手动复制粘贴代码、终端日志和文件内容。\n"
        "OpenCowork 通过以下方式解决这一问题：\n"
        "- 本地代理能力：智能体可以在您的许可下直接读写文件并执行 Shell 命令。\n"
        "- 上下文感知：无需再手动喂数据。智能体会自主探索您的代码库和日志。\n"
        "- 任务编排：复杂的任务（如“重构此模块并更新测试”）会被拆解并由专门的子智能体处理。\n"
        "- 人在回路：通过透明的工具调用审批系统，您始终拥有最终控制权。"
      "Tags" [plain @15:1..15:5]: sequence [block]
        - "人工智能" [plain @16:3..16:7]
        - "大语言模型" [plain @17:3..17:8]
        - "智能体" [plain @18:3..18:6]
        - "自主智能" [plain @19:3..19:7]
      "ReleaseNotesUrl" [plain @20:1..20:16]: "https://github.com/AIDotNet/OpenCowork/releases/tag/0.9.80" [plain @20:18..20:76]
      "ManifestType" [plain @21:1..21:13]: "locale" [plain @21:15..21:21]
      "ManifestVersion" [plain @22:1..22:16]: "1.12.0" [plain @22:18..22:24]
    "#);
}
