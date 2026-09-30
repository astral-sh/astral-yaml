#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-awx-collection-tools-vars-resolution-534b7690"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "name_to_id_endpoint_resolution" [plain @2:1..2:31]: mapping [block]
        "webhook_credential" [plain @3:3..3:21]: "credentials" [plain @3:23..3:34]
        "credential" [plain @4:3..4:13]: "credentials" [plain @4:15..4:26]
        "credential_type" [plain @5:3..5:18]: "credential_types" [plain @5:20..5:36]
        "default_environment" [plain @6:3..6:22]: "execution_environments" [plain @6:24..6:46]
        "execution_environment" [plain @7:3..7:24]: "execution_environments" [plain @7:26..7:48]
        "inventory" [plain @8:3..8:12]: "inventories" [plain @8:14..8:25]
        "organization" [plain @9:3..9:15]: "organizations" [plain @9:17..9:30]
        "project" [plain @10:3..10:10]: "projects" [plain @10:12..10:20]
        "source_project" [plain @11:3..11:17]: "projects" [plain @11:19..11:27]
        "team" [plain @12:3..12:7]: "teams" [plain @12:9..12:14]
        "unified_job_template" [plain @13:3..13:23]: "unified_job_templates" [plain @13:25..13:46]
        "user" [plain @14:3..14:7]: "users" [plain @14:9..14:14]
    "#);
}
