#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-web-config-658cc5d5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "tls_server_config" [plain @4:1..4:18]: mapping [block]
        "cert_file" [plain @5:3..5:12]: "server.crt" [plain @5:14..5:24]
        "key_file" [plain @6:3..6:11]: "server.key" [plain @6:13..6:23]
      "basic_auth_users" [plain @10:1..10:17]: mapping [block]
        "alice" [plain @11:3..11:8]: "$2y$10$mDwo.lAisC94iLAyP81MCesa29IzH37oigHC/42V2pdJlUprsJPze" [plain @11:10..11:70]
        "bob" [plain @12:3..12:6]: "$2y$10$hLqFl9jSjoAAy95Z/zw8Ye8wkdMBM8c5Bn1ptYqP/AXyV0.oy0S8m" [plain @12:8..12:68]
    "#);
}
