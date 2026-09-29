#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-prometheus-eureka-5068b94f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "scrape_configs" [plain @3:1..3:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @6:5..6:13]: "prometheus" [single-quoted @6:15..6:27]
          "static_configs" [plain @7:5..7:19]: sequence [block]
            - mapping [block]
              "targets" [plain @8:9..8:16]: sequence [flow]
                - "localhost:9090" [single-quoted @8:19..8:35]
        - mapping [block]
          "job_name" [plain @11:5..11:13]: "eureka" [single-quoted @11:15..11:23]
          "eureka_sd_configs" [plain @14:5..14:22]: sequence [block]
            - mapping [block]
              "server" [plain @15:9..15:15]: "http://localhost:8761/eureka" [plain @15:17..15:45]
          "relabel_configs" [plain @17:5..17:20]: "" [plain @17:21..17:21]
    "#);
}
