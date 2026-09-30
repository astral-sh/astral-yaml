#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-sigma-component-11-customized-wo-0940e5f3"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_sigma.SigmaComponent" [plain @1:7..1:35]
      "attributes" [plain @3:1..3:11]: mapping [block]
        "organization" [plain @4:3..4:15]: mapping [block]
          "base_url" [plain @5:5..5:13]: "{{ env.SIGMA_BASE_URL }}" [double-quoted @5:15..5:41]
          "client_id" [plain @6:5..6:14]: "{{ env.SIGMA_CLIENT_ID }}" [double-quoted @6:16..6:43]
          "client_secret" [plain @7:5..7:18]: "{{ env.SIGMA_CLIENT_SECRET }}" [double-quoted @7:20..7:51]
        "translation" [plain @8:3..8:14]: mapping [block]
          "for_workbook" [plain @9:5..9:17]: mapping [block]
            "tags" [plain @10:7..10:11]: mapping [block]
              "is_workbook" [plain @11:9..11:20]: "true" [double-quoted @11:22..11:28]
    "#);
}
