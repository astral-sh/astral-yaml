#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("squidfunk-mkdocs-material-docs-blog-authors-73117909"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "authors" [plain @1:1..1:8]: mapping [block]
        "squidfunk" [plain @2:3..2:12]: mapping [block]
          "name" [plain @3:5..3:9]: "Martin Donath" [plain @3:11..3:24]
          "description" [plain @4:5..4:16]: "Creator" [plain @4:18..4:25]
          "avatar" [plain @5:5..5:11]: "https://avatars.githubusercontent.com/u/932156" [plain @5:13..5:59]
          "url" [plain @6:5..6:8]: "https://github.com/squidfunk" [plain @6:10..6:38]
        "alexvoss" [plain @7:3..7:11]: mapping [block]
          "name" [plain @8:5..8:9]: "Alex Voss" [plain @8:11..8:20]
          "description" [plain @9:5..9:16]: "Solutions Architect" [plain @9:18..9:37]
          "avatar" [plain @10:5..10:11]: "https://avatars.githubusercontent.com/u/4134224" [plain @10:13..10:60]
          "url" [plain @11:5..11:8]: "https://github.com/alexvoss" [plain @11:10..11:37]
        "pawamoy" [plain @12:3..12:10]: mapping [block]
          "name" [plain @13:5..13:9]: "Timothée Mazzucotelli" [plain @13:11..13:32]
          "description" [plain @14:5..14:16]: "Developer" [plain @14:18..14:27]
          "avatar" [plain @15:5..15:11]: "https://avatars.githubusercontent.com/u/3999221" [plain @15:13..15:60]
          "url" [plain @16:5..16:8]: "https://github.com/pawamoy" [plain @16:10..16:36]
        "katharinalisalin" [plain @17:3..17:19]: mapping [block]
          "name" [plain @18:5..18:9]: "Kathi" [plain @18:11..18:16]
          "description" [plain @19:5..19:16]: "Communications" [plain @19:18..19:32]
          "avatar" [plain @20:5..20:11]: "https://avatars.githubusercontent.com/u/105352741" [plain @20:13..20:62]
          "url" [plain @21:5..21:8]: "https://github.com/katharinalisalin" [plain @21:10..21:45]
    "#);
}
