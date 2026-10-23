#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("argoproj-argo-workflows-examples-resource-flags-269f895e"), @r#"
       |
    46 |       flags: [
    47 |         "--validate=false"  # validation will be turned off
    48 |       ]
       |       ^ flow continuation must be indented
    "#);
}
