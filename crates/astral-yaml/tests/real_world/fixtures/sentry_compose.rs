#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("sentry-compose"), @r#"
    sentry-compose.yaml:302:7: flow continuation must be indented
    bytes 11055..11056
     300 |           # https://github.com/getsentry/self-hosted/issues/1537
     301 |           "http_proxy='' wget -q -O- http://seaweedfs:9340/healthz http://seaweedfs:9333/cluster/healthz http://seaweedfs:8333/healthz || exit 1",
     302 |       ]
         |       ^
     303 |       interval: 30s
    "#);
}
