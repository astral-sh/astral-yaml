#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-vendoring-vendor-vendor2-1c3932a3"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "atmos/v1" [plain @1:13..1:21]
      "kind" [plain @2:1..2:5]: "AtmosVendorConfig" [plain @2:7..2:24]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "vendor-test-2" [plain @4:9..4:22]
        "description" [plain @5:3..5:14]: literal [@5:16..8:1]
          "Demo vendor configuration showcasing the structure and usage of Atmos vendor configs.\n"
          "This example demonstrates component sourcing from GitHub repositories with versioning.\n"
      "spec" [plain @8:1..8:5]: mapping [block]
        "sources" [plain @9:3..9:10]: sequence [block]
          - mapping [block]
            "component" [plain @10:7..10:16]: "weather" [double-quoted @10:18..10:27]
            "source" [plain @11:7..11:13]: "github.com/cloudposse/atmos.git//examples/demo-library/{{ .Component }}?ref={{.Version}}" [double-quoted @11:15..11:105]
            "version" [plain @12:7..12:14]: "main" [double-quoted @12:16..12:22]
            "targets" [plain @13:7..13:14]: sequence [block]
              - "components/terraform/{{ .Component }}/{{.Version}}" [double-quoted @14:11..14:63]
            "tags" [plain @15:7..15:11]: sequence [block]
              - "demo" [plain @16:11..16:15]
    "#);
}
