#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-demo-services-0dc1b4d7"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "randomize_device_tracker_data" [plain @1:1..1:30]: "" [plain @1:31..1:31]
    "#);
}
