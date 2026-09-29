#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-functional-tests-res-comments-7b676400"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "lorem" [plain @2:1..2:6]: "ipsum" [plain @2:8..2:13]
      "dolor" [plain @4:1..4:6]: "sit" [plain @4:8..4:11]
    "#);
}
