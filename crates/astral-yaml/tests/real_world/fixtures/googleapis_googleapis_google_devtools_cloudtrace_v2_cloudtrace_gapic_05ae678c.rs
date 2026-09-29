#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-devtools-cloudtrace-v2-cloudtrace-gapic-05ae678c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @4:1..4:5]: "com.google.api.codegen.ConfigProto" [plain @4:7..4:41]
      "config_schema_version" [plain @5:1..5:22]: "2.0.0" [plain @5:24..5:29]
      "language_settings" [plain @6:1..6:18]: mapping [block]
        "java" [plain @7:3..7:7]: mapping [block]
          "package_name" [plain @8:5..8:17]: "com.google.cloud.trace.v2" [plain @8:19..8:44]
    "#);
}
