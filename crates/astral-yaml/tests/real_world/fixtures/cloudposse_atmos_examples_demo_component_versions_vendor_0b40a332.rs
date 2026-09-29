#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-component-versions-vendor-0b40a332"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "atmos/v1" [plain @1:13..1:21]
      "kind" [plain @2:1..2:5]: "AtmosVendorConfig" [plain @2:7..2:24]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "demo-yaml-anchors" [plain @4:9..4:26]
        "description" [plain @5:3..5:14]: "Example of using YAML anchors with an Atmos vendoring manifest" [plain @5:16..5:78]
      "spec" [plain @6:1..6:5]: mapping [block]
        "imports" [plain @8:3..8:10]: sequence [flow]
        "bases" [plain @9:3..9:8]: sequence [block]
          - mapping [block] &library
            "source" [plain @11:5..11:11]: "github.com/cloudposse/atmos.git//examples/demo-library/{{ .Component }}?ref={{.Version}}" [double-quoted @11:13..11:103]
            "version" [plain @12:5..12:12]: "main" [double-quoted @12:14..12:20]
            "targets" [plain @13:5..13:12]: sequence [block]
              - "components/terraform/{{ .Component }}/{{.Version}}" [double-quoted @14:9..14:61]
            "included_paths" [plain @15:5..15:19]: sequence [block]
              - "**/*.tf" [double-quoted @16:9..16:18]
              - "**/*.tfvars" [double-quoted @17:9..17:22]
              - "**/*.md" [double-quoted @18:9..18:18]
            "tags" [plain @19:5..19:9]: sequence [block]
              - "demo" [plain @20:9..20:13]
          - mapping [block] &main
            "<<" [plain @23:5..23:7]: *library [@23:9..23:17]
            "version" [plain @24:5..24:12]: "main" [double-quoted @24:14..24:20]
        "sources" [plain @30:3..30:10]: sequence [block]
          - mapping [block]
            "<<" [plain @38:5..38:7]: *main [@38:9..38:14]
            "component" [plain @39:5..39:14]: "ipinfo" [double-quoted @39:16..39:24]
    "#);
}
