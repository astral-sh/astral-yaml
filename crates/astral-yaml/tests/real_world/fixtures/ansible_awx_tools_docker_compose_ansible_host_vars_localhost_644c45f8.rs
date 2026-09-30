#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-host-vars-localhost-644c45f8"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "sources_dest" [plain @2:1..2:13]: "../_sources" [single-quoted @2:15..2:28]
    "#);
}
