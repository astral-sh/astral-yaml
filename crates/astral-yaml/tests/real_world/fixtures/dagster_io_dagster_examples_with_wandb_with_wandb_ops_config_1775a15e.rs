#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-with-wandb-with-wandb-ops-config-1775a15e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "resources" [plain @1:1..1:10]: mapping [block]
        "wandb_config" [plain @2:3..2:15]: mapping [block]
          "config" [plain @3:5..3:11]: mapping [block]
            "entity" [plain @4:7..4:13]: "my_entity" [plain @4:15..4:24]
            "project" [plain @5:7..5:14]: "my_project" [plain @5:16..5:26]
      "ops" [plain @6:1..6:4]: mapping [block]
        "run_launch_agent" [plain @7:3..7:19]: mapping [block]
          "config" [plain @8:5..8:11]: mapping [block]
            "max_jobs" [plain @9:7..9:15]: "-1" [plain @9:17..9:19]
            "queues" [plain @10:7..10:13]: sequence [block]
              - "my_dagster_queue" [plain @11:11..11:27]
        "my_launched_job" [plain @12:3..12:18]: mapping [block]
          "config" [plain @13:5..13:11]: mapping [block]
            "entry_point" [plain @14:7..14:18]: sequence [block]
              - "python" [plain @15:11..15:17]
              - "train.py" [plain @16:11..16:19]
            "queue" [plain @17:7..17:12]: "my_dagster_queue" [plain @17:14..17:30]
            "uri" [plain @18:7..18:10]: "https://github.com/wandb/example-dagster-integration-with-launch" [plain @18:12..18:76]
    "#);
}
