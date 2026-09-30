#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-vars-main-b8e57d5b"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "es_package_url" [plain @2:1..2:15]: "https://artifacts.elastic.co/downloads/elasticsearch/elasticsearch" [double-quoted @2:17..2:85]
      "reserved_xpack_users" [plain @3:1..3:21]: sequence [flow]
        - "elastic" [double-quoted @3:24..3:33]
        - "kibana" [double-quoted @3:34..3:42]
        - "logstash_system" [double-quoted @3:43..3:60]
      "sysd_config_file" [plain @4:1..4:17]: "/etc/systemd/system/elasticsearch.service.d/override.conf" [double-quoted @4:19..4:78]
    "#);
}
