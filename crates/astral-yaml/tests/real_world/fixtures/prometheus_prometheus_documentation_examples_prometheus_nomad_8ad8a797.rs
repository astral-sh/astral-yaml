#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-prometheus-nomad-8ad8a797"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "scrape_configs" [plain @11:1..11:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @13:5..13:13]: "prometheus" [double-quoted @13:15..13:27]
          "static_configs" [plain @14:5..14:19]: sequence [block]
            - mapping [block]
              "targets" [plain @15:9..15:16]: sequence [flow]
                - "localhost:9090" [double-quoted @15:19..15:35]
        - mapping [block]
          "job_name" [plain @18:5..18:13]: "nomad_sd" [single-quoted @18:15..18:25]
          "nomad_sd_configs" [plain @19:5..19:21]: sequence [block]
            - mapping [block]
              "server" [plain @20:9..20:15]: "http://localhost:4646" [single-quoted @20:17..20:40]
          "relabel_configs" [plain @21:5..21:20]: sequence [block]
            - mapping [block]
              "source_labels" [plain @22:9..22:22]: sequence [flow]
                - "__meta_nomad_service" [plain @22:25..22:45]
              "target_label" [plain @23:9..23:21]: "job" [plain @23:23..23:26]
    "#);
}
