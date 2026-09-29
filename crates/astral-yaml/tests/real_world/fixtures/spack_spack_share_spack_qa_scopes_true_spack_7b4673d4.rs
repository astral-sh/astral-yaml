#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-qa-scopes-true-spack-7b4673d4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "spack" [plain @1:1..1:6]: mapping [block]
        "concretizer" [plain @2:3..2:14]: mapping [block]
          "unify" [plain @3:5..3:10]: "true" [plain @3:12..3:16]
    "#);
}
