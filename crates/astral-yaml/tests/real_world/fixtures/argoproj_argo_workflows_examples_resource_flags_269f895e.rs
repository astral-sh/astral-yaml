#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("argoproj-argo-workflows-examples-resource-flags-269f895e"), @r#"
    argoproj-argo-workflows-examples-resource-flags-269f895e.yaml:48:7: flow continuation must be indented
    bytes 1339..1340
      46 |       flags: [
      47 |         "--validate=false"  # validation will be turned off
      48 |       ]
         |       ^
      49 |       manifest: |
    "#);
}
