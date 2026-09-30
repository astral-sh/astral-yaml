#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-vars-debian-85c80066"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "java_version" [plain @2:1..2:13]: "{% if ansible_os_family == 'Debian' and ansible_distribution_major_version == '10' %}11{% else %}8{% endif %}" [double-quoted @2:15..2:126]
      "java" [plain @3:1..3:5]: "{% if es_java is defined %}{{es_java}}{% else %}openjdk-{{ java_version }}-jre-headless{% endif %}" [double-quoted @3:7..3:107]
      "default_file" [plain @4:1..4:13]: "/etc/default/elasticsearch" [double-quoted @4:15..4:43]
      "es_home" [plain @5:1..5:8]: "/usr/share/elasticsearch" [double-quoted @5:10..5:36]
      "es_apt_key_id" [plain @6:1..6:14]: "46095ACC8548582C1A2699A9D27D666CD88E42B4" [double-quoted @6:16..6:58]
    "#);
}
