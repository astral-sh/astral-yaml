#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("devicetree-cpus"), @r##"
    document 1 (start: ---, end: ...)
    directive %YAML "1.2" [@13:1..13:10]
    mapping [block]
      "$id" [plain @15:1..15:4]: "http://devicetree.org/schemas/cpus.yaml#" [plain @15:6..15:46]
      "$schema" [plain @16:1..16:8]: "http://devicetree.org/meta-schemas/base.yaml#" [plain @16:10..16:55]
      "title" [plain @18:1..18:6]: "Common CPU Nodes properties" [plain @18:8..18:35]
      "maintainers" [plain @20:1..20:12]: sequence [block]
        - "Devicetree Specification Mailing List <devicetree-spec@vger.kernel.org>" [plain @21:5..21:76]
      "description" [plain @23:1..23:12]: literal [@23:14..32:1]
        "In Devicetree data files, the layout of CPUs is described in the\n"
        "\"cpus\" node.  This node in turn contains a number of subnodes\n"
        "representing CPUs, which define properties for every cpu.\n"
        "\n"
        "Bindings for CPU nodes follow the Devicetree Specification, available from:\n"
        "\n"
        "https://www.devicetree.org/specifications/\n"
        "\n"
      "properties" [plain @32:1..32:11]: mapping [block]
        "$nodename" [plain @33:3..33:12]: mapping [block]
          "const" [plain @34:5..34:10]: "cpus" [plain @34:12..34:16]
          "description" [plain @35:5..35:16]: "A /cpus node is required for all Devicetrees.  It does not represent a real device in the system, but acts as a container for child \"cpu\" nodes which represent the systems' CPUs." [plain @36:7..38:47]
        "#address-cells" [single-quoted @40:3..40:19]: mapping [block]
          "minimum" [plain @41:5..41:12]: "1" [plain @41:14..41:15]
          "description" [plain @42:5..42:16]: "The value specifies how many cells each element of the reg property array takes in children of this node." [plain @43:7..44:44]
        "#size-cells" [single-quoted @46:3..46:16]: mapping [block]
          "const" [plain @47:5..47:10]: "0" [plain @47:12..47:13]
        "cpu-map" [plain @49:3..49:10]: mapping [block]
          "$ref" [plain @50:5..50:9]: "/schemas/cpu-map.yaml#" [plain @50:11..50:33]
      "patternProperties" [plain @52:1..52:18]: mapping [block]
        "^cpu@[0-9a-f]+$" [single-quoted @53:3..53:20]: mapping [block]
          "$ref" [plain @54:5..54:9]: "cpu.yaml#" [plain @54:11..54:20]
      "required" [plain @56:1..56:9]: sequence [block]
        - "#address-cells" [single-quoted @57:5..57:21]
        - "#size-cells" [single-quoted @58:5..58:18]
      "additionalProperties" [plain @60:1..60:21]: "true" [plain @60:23..60:27]
    "##);
}
