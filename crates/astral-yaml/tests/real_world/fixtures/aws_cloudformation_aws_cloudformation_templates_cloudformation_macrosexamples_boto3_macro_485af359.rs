#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-cloudformation-macrosexamples-boto3-macro-485af359"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "Transform" [plain @1:1..1:10]: "AWS::Serverless-2016-10-31" [plain @1:12..1:38]
      "Resources" [plain @3:1..3:10]: mapping [block]
        "ResourceFunction" [plain @4:3..4:19]: mapping [block]
          "Type" [plain @5:5..5:9]: "AWS::Serverless::Function" [plain @5:11..5:36]
          "Properties" [plain @6:5..6:15]: mapping [block]
            "Runtime" [plain @7:7..7:14]: "python3.11" [plain @7:16..7:26]
            "CodeUri" [plain @8:7..8:14]: "lambda" [plain @8:16..8:22]
            "Handler" [plain @9:7..9:14]: "resource.handler" [plain @9:16..9:32]
            "Policies" [plain @10:7..10:15]: "PowerUserAccess" [plain @10:17..10:32]
        "MacroFunction" [plain @12:3..12:16]: mapping [block]
          "Type" [plain @13:5..13:9]: "AWS::Serverless::Function" [plain @13:11..13:36]
          "Metadata" [plain @14:5..14:13]: mapping [block]
            "guard" [plain @15:7..15:12]: mapping [block]
              "SuppressedRules" [plain @16:9..16:24]: sequence [block]
                - "LAMBDA_INSIDE_VPC" [plain @17:13..17:30]
                - "LAMBDA_FUNCTION_PUBLIC_ACCESS_PROHIBITED" [plain @18:13..18:53]
          "Properties" [plain @19:5..19:15]: mapping [block]
            "Runtime" [plain @20:7..20:14]: "python3.11" [plain @20:16..20:26]
            "CodeUri" [plain @21:7..21:14]: "lambda" [plain @21:16..21:22]
            "Handler" [plain @22:7..22:14]: "macro.handler" [plain @22:16..22:29]
            "Environment" [plain @23:7..23:18]: mapping [block]
              "Variables" [plain @24:9..24:18]: mapping [block]
                "LAMBDA_ARN" [plain @25:11..25:21]: "ResourceFunction.Arn" [plain @25:31..25:51] !<!GetAtt>
        "Macro" [plain @27:3..27:8]: mapping [block]
          "Type" [plain @28:5..28:9]: "AWS::CloudFormation::Macro" [plain @28:11..28:37]
          "Properties" [plain @29:5..29:15]: mapping [block]
            "Name" [plain @30:7..30:11]: "Boto3" [plain @30:13..30:18]
            "FunctionName" [plain @31:7..31:19]: "MacroFunction.Arn" [plain @31:29..31:46] !<!GetAtt>
    "#);
}
