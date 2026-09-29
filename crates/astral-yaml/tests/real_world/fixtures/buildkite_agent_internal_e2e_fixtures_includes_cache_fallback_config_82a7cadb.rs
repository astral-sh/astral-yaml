#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("buildkite-agent-internal-e2e-fixtures-includes-cache-fallback-config-82a7cadb"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "caches" [plain @1:1..1:7]: sequence [block]
        - mapping [block]
          "name" [plain @2:5..2:9]: "with_fallback" [plain @2:11..2:24]
          "cache_key" [plain @3:5..3:14]: sequence [block]
            - "cache-e2e-fallback" [plain @4:9..4:27]
            - mapping [flow]
              "env" [plain @5:11..5:14]: "BUILDKITE_BUILD_ID" [plain @5:16..5:34]
              "fallback_limit" [plain @5:36..5:50]: "true" [plain @5:52..5:56]
            - mapping [flow]
              "env" [plain @6:11..6:14]: "CACHE_VARIANT" [plain @6:16..6:29]
          "target_paths" [plain @7:5..7:17]: sequence [block]
            - "fallback-src" [plain @8:9..8:21]
        - mapping [block]
          "name" [plain @9:5..9:9]: "without_fallback" [plain @9:11..9:27]
          "cache_key" [plain @10:5..10:14]: sequence [block]
            - "cache-e2e-nofallback" [plain @11:9..11:29]
            - mapping [flow]
              "env" [plain @12:11..12:14]: "BUILDKITE_BUILD_ID" [plain @12:16..12:34]
            - mapping [flow]
              "env" [plain @13:11..13:14]: "CACHE_VARIANT" [plain @13:16..13:29]
          "target_paths" [plain @14:5..14:17]: sequence [block]
            - "nofallback-src" [plain @15:9..15:23]
    "#);
}
