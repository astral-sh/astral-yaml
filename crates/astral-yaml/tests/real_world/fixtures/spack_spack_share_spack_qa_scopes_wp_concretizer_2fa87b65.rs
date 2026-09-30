#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-qa-scopes-wp-concretizer-2fa87b65"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "concretizer" [plain @1:1..1:12]: mapping [block]
        "unify" [plain @2:3..2:8]: "when_possible" [double-quoted @2:10..2:25]
    "#);
}
