#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("espanso-combining-characters-manifest"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "author" [plain @1:1..1:7]: "José Ferreira" [double-quoted @1:9..1:27]
      "description" [plain @2:1..2:12]: "Modifies characters with a collection of symbols e.g.: à, â, a\u{20d7}, a\u{305}, a\u{20dc}" [double-quoted @2:14..3:22]
      "name" [plain @4:1..4:5]: "combining-characters" [plain @4:7..4:27]
      "title" [plain @5:1..5:6]: "Letter modifiers" [plain @5:8..5:24]
      "version" [plain @6:1..6:8]: "0.2.0" [plain @6:10..6:15]
      "homepage" [plain @7:1..7:9]: "https://github.com/jpmvferreira/espanso-mega-pack" [plain @7:11..7:60]
      "tags" [plain @8:1..8:5]: sequence [flow]
        - "symbols" [double-quoted @8:8..8:17]
        - "math" [double-quoted @8:19..8:25]
    "#);
}
