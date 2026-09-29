#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-vars-redhat-0705b60e"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "java" [plain @2:1..2:5]: "{{ es_java | default('java-1.8.0-openjdk.x86_64') }}" [double-quoted @2:7..2:61]
      "default_file" [plain @3:1..3:13]: "/etc/sysconfig/elasticsearch" [double-quoted @3:15..3:45]
      "es_home" [plain @4:1..4:8]: "/usr/share/elasticsearch" [double-quoted @4:10..4:36]
    "#);
}
