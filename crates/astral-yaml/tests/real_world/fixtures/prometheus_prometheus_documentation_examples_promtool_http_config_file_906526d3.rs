#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-promtool-http-config-file-906526d3"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "basic_auth" [plain @6:1..6:11]: mapping [block]
        "username" [plain @7:3..7:11]: "alice" [plain @7:13..7:18]
        "password" [plain @8:3..8:11]: "verylongpassword" [plain @8:13..8:29]
      "tls_config" [plain @10:1..10:11]: mapping [block]
        "cert_file" [plain @11:3..11:12]: "server.crt" [plain @11:14..11:24]
        "key_file" [plain @12:3..12:11]: "server.key" [plain @12:13..12:23]
    "#);
}
