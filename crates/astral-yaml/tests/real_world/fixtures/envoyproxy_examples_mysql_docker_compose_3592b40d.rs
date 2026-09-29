#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-mysql-docker-compose-3592b40d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "proxy" [plain @3:3..3:8]: mapping [block]
          "build" [plain @4:5..4:10]: mapping [block]
            "context" [plain @5:7..5:14]: "." [plain @5:16..5:17]
            "dockerfile" [plain @6:7..6:17]: "../shared/envoy/Dockerfile" [plain @6:19..6:45]
            "args" [plain @7:7..7:11]: mapping [block]
              "ENVOY_VARIANT" [plain @8:9..8:22]: "contrib-dev" [plain @8:24..8:35]
          "depends_on" [plain @9:5..9:15]: mapping [block]
            "mysql" [plain @10:7..10:12]: mapping [block]
              "condition" [plain @11:9..11:18]: "service_started" [plain @11:20..11:35]
          "ports" [plain @12:5..12:10]: sequence [block]
            - "${PORT_ADMIN:-8001}:8001" [double-quoted @13:7..13:33]
        "mysql" [plain @15:3..15:8]: mapping [block]
          "build" [plain @16:5..16:10]: mapping [block]
            "context" [plain @17:7..17:14]: "." [plain @17:16..17:17]
            "dockerfile" [plain @18:7..18:17]: "Dockerfile-mysql" [plain @18:19..18:35]
          "environment" [plain @19:5..19:16]: sequence [block]
            - "MYSQL_ALLOW_EMPTY_PASSWORD=yes" [plain @20:7..20:37]
    "#);
}
