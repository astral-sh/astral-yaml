#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("sentry-compose"), @r#"
        |
    300 |           # https://github.com/getsentry/self-hosted/issues/1537
    301 |           "http_proxy='' wget -q -O- http://seaweedfs:9340/healthz http://seaweedfs:9333/cluster/healthz http:/...
    302 |       ]
        |       ^ flow continuation must be indented
    "#);
}
