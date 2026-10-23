#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("googleapis-googleapis-google-cloud-functions-v2-cloudfunctions-v2-06aa219c"), @"
       |
    17 |   summary: 'Manages lightweight user-provided functions executed in response to events.'
    18 |   overview: 'Manages lightweight user-provided functions executed in response to
    19 | events.'
       | ^ invalid continuation of quoted scalar
    ");
}
