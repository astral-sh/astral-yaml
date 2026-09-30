#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-cloudformation-macrosexamples-count-template-40978da8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: literal [@3:14..6:1]
        "The Count macro is an iterator for creating multiple resources\n"
      "Transform" [plain @6:1..6:10]: "AWS::Serverless-2016-10-31" [plain @6:12..6:38]
      "Resources" [plain @8:1..8:10]: mapping [block]
        "Macro" [plain @9:3..9:8]: mapping [block]
          "Type" [plain @10:5..10:9]: "AWS::CloudFormation::Macro" [plain @10:11..10:37]
          "Properties" [plain @11:5..11:15]: mapping [block]
            "Name" [plain @12:7..12:11]: "Count" [plain @12:13..12:18]
            "FunctionName" [plain @13:7..13:19]: "CountMacroFunction.Arn" [plain @13:29..13:51] !<!GetAtt>
        "CountMacroFunction" [plain @15:3..15:21]: mapping [block]
          "Type" [plain @16:5..16:9]: "AWS::Serverless::Function" [plain @16:11..16:36]
          "Metadata" [plain @17:5..17:13]: mapping [block]
            "guard" [plain @18:7..18:12]: mapping [block]
              "SuppressedRules" [plain @19:9..19:24]: sequence [block]
                - "LAMBDA_INSIDE_VPC" [plain @20:13..20:30]
                - "LAMBDA_FUNCTION_PUBLIC_ACCESS_PROHIBITED" [plain @21:13..21:53]
          "Properties" [plain @22:5..22:15]: mapping [block]
            "CodeUri" [plain @23:7..23:14]: "src" [plain @23:16..23:19]
            "Handler" [plain @24:7..24:14]: "index.handler" [plain @24:16..24:29]
            "Runtime" [plain @25:7..25:14]: "python3.11" [plain @25:16..25:26]
            "Timeout" [plain @26:7..26:14]: "5" [plain @26:16..26:17]
    "#);
}
