#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-cloud-devicestreaming-v1-devicestreaming-v1-01be1e20"), @r####"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "devicestreaming.googleapis.com" [plain @3:7..3:37]
      "title" [plain @4:1..4:6]: "Device Streaming API" [plain @4:8..4:28]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.cloud.devicestreaming.v1.DirectAccessService" [plain @7:9..7:60]
      "documentation" [plain @9:1..9:14]: mapping [block]
        "summary" [plain @10:3..10:10]: "The Cloud API for device streaming usage." [plain @10:12..10:53]
        "overview" [plain @11:3..11:11]: literal [@11:13..38:1]
          "The Developer Device Platform is a next-generation, commercial-grade\n"
          "platform for remote device orchestration. It provides a managed device\n"
          "farm for developers to build and test applications using remote\n"
          "infrastructure, natively integrating with Google Cloud.\n"
          "\n"
          "The **Device Streaming API** provides real-time, interactive access to\n"
          "remote physical Android devices for live debugging, testing, and streaming\n"
          "across physical device catalogs.\n"
          "\n"
          "### Key Features\n"
          "\n"
          "- **Physical Device Access**: Securely access real physical Android\n"
          "devices—including Google Pixel models and partner devices from Samsung,\n"
          "Xiaomi, OPPO, OnePlus, vivo, and Transsion—to deploy and test applications\n"
          "remotely. - **Direct ADB over SSL**: Establish direct ADB over SSL\n"
          "connections for real-time device interaction (screen manipulation,\n"
          "rotation, unfolding) and native integration with developer tooling,\n"
          "Debugger, Profiler, Logcat, and Device Explorer. - **Automated Security &\n"
          "Privacy**: Automatic data wiping and factory resets are performed after\n"
          "every session to protect developer data and maintain device security. -\n"
          "**Google Cloud Integration**: Fully integrated with Cloud Logging, Cloud\n"
          "IAM, and Google Cloud billing.\n"
          "\n"
          "For more details, see the [Developer Device Platform\n"
          "documentation](https://docs.cloud.google.com/developer-device-platform)."
      "authentication" [plain @38:1..38:15]: mapping [block]
        "rules" [plain @39:3..39:8]: sequence [block]
          - mapping [block]
            "selector" [plain @40:5..40:13]: "google.cloud.devicestreaming.v1.DirectAccessService.*" [single-quoted @40:15..40:70]
            "oauth" [plain @41:5..41:10]: mapping [block]
              "canonical_scopes" [plain @42:7..42:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @42:25..45:1]
      "publishing" [plain @45:1..45:11]: mapping [block]
        "new_issue_uri" [plain @46:3..46:16]: "https://issuetracker.google.com/issues/new?component=1767292&template=2112325" [plain @46:18..46:95]
        "documentation_uri" [plain @47:3..47:20]: "https://cloud.google.com/device-streaming/docs" [plain @47:22..47:68]
        "api_short_name" [plain @48:3..48:17]: "devicestreaming" [plain @48:19..48:34]
        "github_label" [plain @49:3..49:15]: "api: devicestreaming" [single-quoted @49:17..49:39]
        "doc_tag_prefix" [plain @50:3..50:17]: "devicestreaming" [plain @50:19..50:34]
        "organization" [plain @51:3..51:15]: "CLOUD" [plain @51:17..51:22]
        "library_settings" [plain @52:3..52:19]: sequence [block]
          - mapping [block]
            "version" [plain @53:5..53:12]: "google.cloud.devicestreaming.v1" [plain @53:14..53:45]
            "launch_stage" [plain @54:5..54:17]: "GA" [plain @54:19..54:21]
            "java_settings" [plain @55:5..55:18]: mapping [block]
              "common" [plain @56:7..56:13]: mapping [block]
                "destinations" [plain @57:9..57:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @58:11..58:26]
            "cpp_settings" [plain @59:5..59:17]: mapping [block]
              "common" [plain @60:7..60:13]: mapping [block]
                "destinations" [plain @61:9..61:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @62:11..62:26]
            "php_settings" [plain @63:5..63:17]: mapping [block]
              "common" [plain @64:7..64:13]: mapping [block]
                "destinations" [plain @65:9..65:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @66:11..66:26]
            "python_settings" [plain @67:5..67:20]: mapping [block]
              "common" [plain @68:7..68:13]: mapping [block]
                "destinations" [plain @69:9..69:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @70:11..70:26]
            "node_settings" [plain @71:5..71:18]: mapping [block]
              "common" [plain @72:7..72:13]: mapping [block]
                "destinations" [plain @73:9..73:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @74:11..74:26]
            "dotnet_settings" [plain @75:5..75:20]: mapping [block]
              "common" [plain @76:7..76:13]: mapping [block]
                "destinations" [plain @77:9..77:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @78:11..78:26]
            "ruby_settings" [plain @79:5..79:18]: mapping [block]
              "common" [plain @80:7..80:13]: mapping [block]
                "destinations" [plain @81:9..81:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @82:11..82:26]
            "go_settings" [plain @83:5..83:16]: mapping [block]
              "common" [plain @84:7..84:13]: mapping [block]
                "destinations" [plain @85:9..85:21]: sequence [block]
                  - "PACKAGE_MANAGER" [plain @86:11..86:26]
        "rest_reference_documentation_uri" [plain @87:3..87:35]: "https://docs.cloud.google.com/device-streaming/docs/reference/rest" [plain @87:37..87:103]
    "####);
}
