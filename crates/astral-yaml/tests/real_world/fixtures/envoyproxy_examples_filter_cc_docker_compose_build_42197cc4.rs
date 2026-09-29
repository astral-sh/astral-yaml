#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-filter-cc-docker-compose-build-42197cc4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "filter_build" [plain @2:3..2:15]: mapping [block]
          "build" [plain @3:5..3:10]: mapping [block]
            "context" [plain @4:7..4:14]: "../shared/build" [plain @4:16..4:31]
          "command" [plain @5:5..5:12]: sequence [block]
            - "bash" [plain @6:7..6:11]
            - "-c" [plain @7:7..7:9]
            - literal [@8:7..14:1]
              "set -eo pipefail\n"
              "bazel build --disk_cache=/tmp/disk_cache \\\n"
              "   --repository_cache=/tmp/repository_cache --experimental_repository_cache_hardlinks \\\n"
              "   //:envoy\n"
              "cp -Lf bazel-bin/envoy /output/envoy\n"
          "environment" [plain @14:5..14:16]: mapping [block]
            "BUILD_UID" [plain @15:7..15:16]: "${UID:-1000}" [plain @15:18..15:30]
            "GITHUB_TOKEN" [plain @16:7..16:19]: "${GITHUB_TOKEN:-}" [double-quoted @16:21..16:40]
          "entrypoint" [plain @17:5..17:15]: "/source/shared/build/build-entrypoint.sh" [plain @17:17..17:57]
          "working_dir" [plain @18:5..18:16]: "/source/filter-cc" [plain @18:18..18:35]
          "volumes" [plain @19:5..19:12]: sequence [block]
            - "${ENVOY_EXAMPLES_CACHE_DIR:-${HOME}/.cache/envoy/examples}:/home/envoybuild/.cache" [plain @20:7..20:89]
            - "..:/source" [plain @21:7..21:17]
            - "./bin:/output" [plain @22:7..22:20]
        "filter_test" [plain @24:3..24:14]: mapping [block]
          "build" [plain @25:5..25:10]: mapping [block]
            "context" [plain @26:7..26:14]: "../shared/build" [plain @26:16..26:31]
          "command" [plain @27:5..27:12]: sequence [block]
            - "bash" [plain @28:7..28:11]
            - "-c" [plain @29:7..29:9]
            - literal [@30:7..35:1]
              "set -eo pipefail\n"
              "bazel test --disk_cache=/tmp/disk_cache \\\n"
              "   --repository_cache=/tmp/repository_cache --experimental_repository_cache_hardlinks \\\n"
              "   //:http_filter_integration_test\n"
          "environment" [plain @35:5..35:16]: mapping [block]
            "BUILD_UID" [plain @36:7..36:16]: "${UID:-1000}" [plain @36:18..36:30]
            "GITHUB_TOKEN" [plain @37:7..37:19]: "${GITHUB_TOKEN:-}" [double-quoted @37:21..37:40]
          "entrypoint" [plain @38:5..38:15]: "/source/shared/build/build-entrypoint.sh" [plain @38:17..38:57]
          "working_dir" [plain @39:5..39:16]: "/source/filter-cc" [plain @39:18..39:35]
          "volumes" [plain @40:5..40:12]: sequence [block]
            - "${ENVOY_EXAMPLES_CACHE_DIR:-${HOME}/.cache/envoy/examples}:/home/envoybuild/.cache" [plain @41:7..41:89]
            - "..:/source" [plain @42:7..42:17]
            - "./bin:/output" [plain @43:7..43:20]
    "#);
}
