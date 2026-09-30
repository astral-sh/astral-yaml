#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-prometheus-agent-b418f284"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "global" [plain @2:1..2:7]: mapping [block]
        "scrape_interval" [plain @3:3..3:18]: "15s" [plain @3:20..3:23]
        "evaluation_interval" [plain @4:3..4:22]: "15s" [plain @4:24..4:27]
      "scrape_configs" [plain @9:1..9:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @11:5..11:13]: "prometheus" [double-quoted @11:15..11:27]
          "static_configs" [plain @16:5..16:19]: sequence [block]
            - mapping [block]
              "targets" [plain @17:9..17:16]: sequence [flow]
                - "localhost:9090" [double-quoted @17:19..17:35]
      "remote_write" [plain @20:1..20:13]: sequence [block]
        - mapping [block]
          "url" [plain @22:5..22:8]: "http://remote-write-url" [double-quoted @22:10..22:35]
    "#);
}
