#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("unity-editor-build-settings"), @r#"
    document 1 (start: ---, end: implicit)
    directive %YAML "1.1" [@1:1..1:10]
    directive %TAG "!u! tag:unity3d.com,2011:" [@2:1..2:31]
    mapping [block] &1 !<tag:unity3d.com,2011:1045>
      "EditorBuildSettings" [plain @4:1..4:20]: mapping [block]
        "m_ObjectHideFlags" [plain @5:3..5:20]: "0" [plain @5:22..5:23]
        "serializedVersion" [plain @6:3..6:20]: "2" [plain @6:22..6:23]
        "m_Scenes" [plain @7:3..7:11]: sequence [flow]
        "m_configObjects" [plain @8:3..8:18]: mapping [flow]
    "#);
}
