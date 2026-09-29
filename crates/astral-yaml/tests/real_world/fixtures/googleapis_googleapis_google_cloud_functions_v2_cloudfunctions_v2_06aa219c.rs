#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("googleapis-googleapis-google-cloud-functions-v2-cloudfunctions-v2-06aa219c"), @"
    googleapis-googleapis-google-cloud-functions-v2-cloudfunctions-v2-06aa219c.yaml:19:1: invalid continuation of quoted scalar
    bytes 570..571
      17 |   summary: 'Manages lightweight user-provided functions executed in response to events.'
      18 |   overview: 'Manages lightweight user-provided functions executed in response to
      19 | events.'
         | ^
      20 |   rules:
    ");
}
