#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("nuclei-directory-listing"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "id" [plain @1:1..1:3]: "directory-listing-no-host-header" [plain @1:5..1:37]
      "info" [plain @3:1..3:5]: mapping [block]
        "name" [plain @4:3..4:7]: "Directory Listing - No Host header" [plain @4:9..4:43]
        "author" [plain @5:3..5:9]: "kazet" [plain @5:11..5:16]
        "severity" [plain @6:3..6:11]: "unknown" [plain @6:13..6:20]
        "description" [plain @7:3..7:14]: literal [@7:16..9:1]
          "The HTTP server is configured to list files in the root directory when no Host header is provided.\n"
        "metadata" [plain @9:3..9:11]: mapping [block]
          "verified" [plain @10:5..10:13]: "true" [plain @10:15..10:19]
          "max-request" [plain @11:5..11:16]: "1" [plain @11:18..11:19]
        "tags" [plain @12:3..12:7]: "misconfig,listing,vuln" [plain @12:9..12:31]
      "flow" [plain @14:1..14:5]: "http(1) && http(2)" [plain @14:7..14:25]
      "http" [plain @16:1..16:5]: sequence [block]
        - mapping [block]
          "raw" [plain @17:5..17:8]: sequence [block]
            - literal [@18:9..22:1]
              "GET / HTTP/1.1\n"
              "Host: {{Hostname}}\n"
          "matchers" [plain @22:5..22:13]: sequence [block]
            - mapping [block]
              "type" [plain @23:9..23:13]: "dsl" [plain @23:15..23:18]
              "dsl" [plain @24:9..24:12]: sequence [block]
                - "!contains_any(body,\"<title>Index of\",\"<title>Directory listing of\")" [single-quoted @25:13..25:82]
              "internal" [plain @26:9..26:17]: "true" [plain @26:19..26:23]
        - mapping [block]
          "raw" [plain @28:5..28:8]: sequence [block]
            - literal [@29:9..33:1]
              "@Host: {{Hostname}}\n"
              "GET / HTTP/1.0\n"
              "\n"
          "unsafe" [plain @33:5..33:11]: "true" [plain @33:13..33:17]
          "host-redirects" [plain @35:5..35:19]: "true" [plain @35:21..35:25]
          "max-redirects" [plain @36:5..36:18]: "2" [plain @36:20..36:21]
          "matchers" [plain @38:5..38:13]: sequence [block]
            - mapping [block]
              "type" [plain @39:9..39:13]: "dsl" [plain @39:15..39:18]
              "dsl" [plain @40:9..40:12]: sequence [block]
                - "contains_any(body,\"<title>Index of\",\"<title>Directory listing of\")" [single-quoted @41:13..41:81]
                - "status_code == 200" [single-quoted @42:13..42:33]
              "condition" [plain @43:9..43:18]: "and" [plain @43:20..43:23]
    "#);
}
