#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-ambient-argo-tag-chart-values-242b342a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "global" [plain @1:1..1:7]: mapping [block]
        "imagePullSecrets" [plain @6:3..6:19]: sequence [flow]
        "istioNamespace" [plain @9:3..9:17]: "istio-system" [plain @9:19..9:31]
        "istiod" [plain @11:3..11:9]: mapping [block]
          "enableAnalysis" [plain @12:5..12:19]: "false" [plain @12:21..12:26]
        "configValidation" [plain @14:3..14:19]: "true" [plain @14:21..14:25]
        "externalIstiod" [plain @15:3..15:17]: "false" [plain @15:19..15:24]
        "remotePilotAddress" [plain @16:3..16:21]: "" [double-quoted @16:23..16:25]
      "base" [plain @18:1..18:5]: mapping [block]
        "enableCRDTemplates" [plain @20:3..20:21]: "false" [plain @20:23..20:28]
        "validationURL" [plain @24:3..24:16]: "" [double-quoted @24:18..24:20]
        "enableIstioConfigCRDs" [plain @27:3..27:24]: "true" [plain @27:26..27:30]
        "tags" [plain @41:3..41:7]: mapping [flow]
    "#);
}
