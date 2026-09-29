#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-dynamic-config-fs-envoy-3ad2ed0e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "node" [plain @1:1..1:5]: mapping [block]
        "id" [plain @2:3..2:5]: "id_1" [plain @2:7..2:11]
        "cluster" [plain @3:3..3:10]: "test" [plain @3:12..3:16]
      "dynamic_resources" [plain @5:1..5:18]: mapping [block]
        "cds_config" [plain @6:3..6:13]: mapping [block]
          "path_config_source" [plain @7:5..7:23]: mapping [block]
            "path" [plain @8:7..8:11]: "/var/lib/envoy/cds.yaml" [plain @8:13..8:36]
        "lds_config" [plain @9:3..9:13]: mapping [block]
          "path_config_source" [plain @10:5..10:23]: mapping [block]
            "path" [plain @11:7..11:11]: "/var/lib/envoy/lds.yaml" [plain @11:13..11:36]
      "admin" [plain @13:1..13:6]: mapping [block]
        "address" [plain @14:3..14:10]: mapping [block]
          "socket_address" [plain @15:5..15:19]: mapping [block]
            "address" [plain @16:7..16:14]: "0.0.0.0" [plain @16:16..16:23]
            "port_value" [plain @17:7..17:17]: "19000" [plain @17:19..17:24]
    "#);
}
