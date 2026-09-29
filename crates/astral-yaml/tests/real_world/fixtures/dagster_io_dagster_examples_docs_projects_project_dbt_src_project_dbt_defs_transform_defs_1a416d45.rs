#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-projects-project-dbt-src-project-dbt-defs-transform-defs-1a416d45"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_dbt.DbtProjectComponent" [plain @1:7..1:38]
      "template_vars_module" [plain @3:1..3:21]: ".template_vars" [plain @3:23..3:37]
      "attributes" [plain @4:1..4:11]: mapping [block]
        "project" [plain @5:3..5:10]: "{{ context.project_root }}/src/project_dbt/analytics" [single-quoted @5:12..5:66]
        "select" [plain @6:3..6:9]: "tag:daily" [double-quoted @6:11..6:22]
        "translation" [plain @7:3..7:14]: mapping [block]
          "key" [plain @8:5..8:8]: "taxi_{{ node.name }}" [double-quoted @8:10..8:32]
          "group_name" [plain @9:5..9:15]: "dbt" [plain @9:17..9:20]
        "cli_args" [plain @11:3..11:11]: sequence [block]
          - "build" [plain @12:7..12:12]
          - mapping [block]
            "--vars" [plain @13:7..13:13]: "" [plain @13:14..13:14]
            "min_date" [plain @14:7..14:15]: "{{ context.partition_time_window.start.strftime('%Y-%m-%d') }}" [double-quoted @14:17..14:81]
            "max_date" [plain @15:7..15:15]: "{{ context.partition_time_window.end.strftime('%Y-%m-%d') }}" [double-quoted @15:17..15:79]
      "post_processing" [plain @18:1..18:16]: mapping [block]
        "assets" [plain @19:3..19:9]: sequence [block]
          - mapping [block]
            "target" [plain @20:7..20:13]: "*" [double-quoted @20:15..20:18]
            "attributes" [plain @21:7..21:17]: mapping [block]
              "partitions_def" [plain @22:9..22:23]: "{{ daily_partitions_def }}" [double-quoted @22:25..22:53]
    "#);
}
