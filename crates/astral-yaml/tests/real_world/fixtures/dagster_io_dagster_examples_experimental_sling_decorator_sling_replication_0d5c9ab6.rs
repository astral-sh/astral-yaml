#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-experimental-sling-decorator-sling-replication-0d5c9ab6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "source" [plain @1:1..1:7]: "MY_POSTGRES" [plain @1:9..1:20]
      "target" [plain @2:1..2:7]: "MY_DUCKDB" [plain @2:9..2:18]
      "defaults" [plain @4:1..4:9]: mapping [block]
        "mode" [plain @5:3..5:7]: "full-refresh" [plain @5:9..5:21]
        "object" [plain @7:3..7:9]: "{stream_schema}_{stream_table}" [double-quoted @7:11..7:43]
      "streams" [plain @9:1..9:8]: mapping [block]
        "public.accounts" [plain @10:3..10:18]: "" [plain @10:19..10:19]
        "public.users" [plain @11:3..11:15]: mapping [block]
          "disabled" [plain @12:5..12:13]: "true" [plain @12:15..12:19]
        "public.finance_departments_old" [plain @13:3..13:33]: mapping [block]
          "object" [plain @14:5..14:11]: "departments" [double-quoted @14:13..14:26]
          "source_options" [plain @15:5..15:19]: mapping [block]
            "empty_as_null" [plain @16:7..16:20]: "false" [plain @16:22..16:27]
          "meta" [plain @17:5..17:9]: mapping [block]
            "dagster_source" [plain @18:7..18:21]: "boo" [plain @18:23..18:26]
        "public.\"Transactions\"" [plain @20:3..20:24]: mapping [block]
          "mode" [plain @21:5..21:9]: "incremental" [plain @21:11..21:22]
          "primary_key" [plain @22:5..22:16]: "id" [plain @22:18..22:20]
          "update_key" [plain @23:5..23:15]: "last_updated_at" [plain @23:17..23:32]
        "public.all_users" [plain @25:3..25:19]: mapping [block]
          "sql" [plain @26:5..26:8]: literal [@26:10..29:1]
            "select all_user_id, name \n"
            "from public.\"all_Users\"\n"
          "object" [plain @29:5..29:11]: "public.all_users" [plain @29:13..29:29]
      "env" [plain @31:1..31:4]: mapping [block]
        "SLING_LOADED_AT_COLUMN" [plain @32:3..32:25]: "true" [plain @32:27..32:31]
        "SLING_STREAM_URL_COLUMN" [plain @33:3..33:26]: "true" [plain @33:28..33:32]
    "#);
}
