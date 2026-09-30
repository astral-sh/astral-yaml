#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-defaults-include-077bc74b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "include" [plain @1:1..1:8]: sequence [block]
        - mapping [block]
          "path" [plain @3:5..3:9]: "${platform}" [double-quoted @3:11..3:24]
          "optional" [plain @4:5..4:13]: "true" [plain @4:15..4:19]
        - mapping [block]
          "path" [plain @7:5..7:9]: "base" [plain @7:11..7:15]
    "#);
}
