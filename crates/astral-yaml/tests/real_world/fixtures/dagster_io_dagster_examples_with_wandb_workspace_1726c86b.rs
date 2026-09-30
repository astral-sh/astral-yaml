#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-with-wandb-workspace-1726c86b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "load_from" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "python_module" [plain @2:5..2:18]: "with_wandb.definitions" [plain @2:20..2:42]
    "#);
}
