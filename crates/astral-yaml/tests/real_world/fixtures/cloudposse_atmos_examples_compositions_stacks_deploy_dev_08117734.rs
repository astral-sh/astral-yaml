#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-compositions-stacks-deploy-dev-08117734"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "vars" [plain @1:1..1:5]: mapping [block]
        "stage" [plain @3:3..3:8]: "dev" [plain @3:10..3:13]
      "components" [plain @8:1..8:11]: mapping [block]
        "container" [plain @9:3..9:12]: mapping [block]
          "frontend" [plain @10:5..10:13]: mapping [block]
            "composition" [plain @11:7..11:18]: "storefront" [plain @11:20..11:30]
            "image" [plain @12:7..12:12]: "nginx:alpine" [plain @12:14..12:26]
          "api" [plain @14:5..14:8]: mapping [block]
            "composition" [plain @15:7..15:18]: "storefront" [plain @15:20..15:30]
            "image" [plain @16:7..16:12]: "nginx:alpine" [plain @16:14..16:26]
          "database" [plain @18:5..18:13]: mapping [block]
            "composition" [plain @19:7..19:18]: "storefront" [plain @19:20..19:30]
            "image" [plain @20:7..20:12]: "postgres:16-alpine" [plain @20:14..20:32]
    "#);
}
