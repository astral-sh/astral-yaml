#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("sublime-regex-replacement"), @r#"
    document 1 (start: ---, end: implicit)
    directive %YAML "1.2" [@1:1..1:10]
    mapping [block]
      "scope" [plain @5:1..5:6]: "source.regexp-replace" [plain @5:8..5:29]
      "name" [plain @6:1..6:5]: "Regular Expression Replacement" [plain @6:7..6:37]
      "hidden" [plain @7:1..7:7]: "true" [plain @7:9..7:13]
      "contexts" [plain @9:1..9:9]: mapping [block]
        "main" [plain @10:3..10:7]: sequence [block]
          - mapping [block]
            "include" [plain @11:7..11:14]: "escapes" [plain @11:16..11:23]
          - mapping [block]
            "include" [plain @12:7..12:14]: "variables" [plain @12:16..12:25]
        "escapes" [plain @14:3..14:10]: sequence [block]
          - mapping [block]
            "match" [plain @15:7..15:12]: "\\\\[1-9]" [plain @15:14..15:21]
            "scope" [plain @16:7..16:12]: "variable.language.backref.regexp-replace" [plain @16:14..16:54]
          - mapping [block]
            "match" [plain @17:7..17:12]: "\\\\[lLuUE]" [plain @17:14..17:23]
            "scope" [plain @18:7..18:12]: "keyword.operator.case-conversion.regexp-replace" [plain @18:14..18:61]
          - mapping [block]
            "match" [plain @19:7..19:12]: "\\\\[aefnrtv()]" [plain @19:14..19:27]
            "scope" [plain @20:7..20:12]: "constant.character.escape.regexp-replace" [plain @20:14..20:54]
          - mapping [block]
            "match" [plain @21:7..21:12]: "\\\\x(\\h{2}|\\{\\h{4}\\})" [plain @21:14..21:34]
            "scope" [plain @22:7..22:12]: "constant.character.escape.regexp-replace" [plain @22:14..22:54]
          - mapping [block]
            "match" [plain @23:7..23:12]: "\\\\." [plain @23:14..23:17]
            "scope" [plain @24:7..24:12]: "constant.character.escape.regexp-replace" [plain @24:14..24:54]
        "variables" [plain @26:3..26:12]: sequence [block]
          - mapping [block]
            "match" [plain @27:7..27:12]: "\\$\\$" [plain @27:14..27:18]
            "scope" [plain @28:7..28:12]: "constant.character.escape.regexp-replace" [plain @28:14..28:54]
          - mapping [block]
            "match" [plain @29:7..29:12]: literal [@29:14..35:1]
              "\\$(?x:\n"
              "  \\d+ | [`'&] | \\^N | \\+(?: \\{ \\w+ \\} )? |\n"
              "  \\{ (?: \\d+ | \\^(?: MATCH | PREMATCH | POSTMATCH ) ) \\} |\n"
              "  LAST_PAREN_MATCH | MATCH | PREMATCH | POSTMATCH | LAST_SUBMATCH_RESULT\n"
              ")"
            "scope" [plain @35:7..35:12]: "variable.language.match.regexp-replace" [plain @35:14..35:52]
    "#);
}
