#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-sns-snstopic-06f5f13b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Best Practice SNS Topic" [plain @3:14..3:37]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "SubscriptionEndPoint" [plain @6:3..6:23]: mapping [block]
          "Description" [plain @7:5..7:16]: "The endpoint that receives notifications from the Amazon SNS topic. The endpoint value depends on the protocol that you specify. This could be a URL or ARN" [plain @7:18..7:173]
          "Type" [plain @8:5..8:9]: "String" [plain @8:11..8:17]
        "SubscriptionProtocol" [plain @10:3..10:23]: mapping [block]
          "Description" [plain @11:5..11:16]: "The subscription's protocol" [plain @11:18..11:45]
          "Type" [plain @12:5..12:9]: "String" [plain @12:11..12:17]
          "AllowedValues" [plain @13:5..13:18]: sequence [block]
            - "http" [plain @14:9..14:13]
            - "https" [plain @15:9..15:14]
            - "email" [plain @16:9..16:14]
            - "email-json" [plain @17:9..17:19]
            - "sms" [plain @18:9..18:12]
            - "sqs" [plain @19:9..19:12]
            - "application" [plain @20:9..20:20]
            - "lambda" [plain @21:9..21:15]
          "Default" [plain @22:5..22:12]: "sqs" [plain @22:14..22:17]
      "Mappings" [plain @24:1..24:9]: mapping [flow]
      "Conditions" [plain @26:1..26:11]: mapping [flow]
      "Resources" [plain @28:1..28:10]: mapping [block]
        "SNSTopic" [plain @29:3..29:11]: mapping [block]
          "Type" [plain @30:5..30:9]: "AWS::SNS::Topic" [plain @30:11..30:26]
          "Properties" [plain @31:5..31:15]: mapping [flow]
        "SNSSubscription" [plain @33:3..33:18]: mapping [block]
          "Type" [plain @34:5..34:9]: "AWS::SNS::Subscription" [plain @34:11..34:33]
          "Properties" [plain @35:5..35:15]: mapping [block]
            "Endpoint" [plain @36:7..36:15]: "SubscriptionEndPoint" [plain @36:22..36:42] !<!Ref>
            "Protocol" [plain @37:7..37:15]: "SubscriptionProtocol" [plain @37:22..37:42] !<!Ref>
            "TopicArn" [plain @38:7..38:15]: "SNSTopic" [plain @38:22..38:30] !<!Ref>
      "Outputs" [plain @40:1..40:8]: mapping [block]
        "TopicARN" [plain @41:3..41:11]: mapping [block]
          "Description" [plain @42:5..42:16]: "ARN of newly created SNS Topic" [plain @42:18..42:48]
          "Value" [plain @43:5..43:10]: "SNSTopic" [plain @43:17..43:25] !<!Ref>
        "QueueName" [plain @45:3..45:12]: mapping [block]
          "Description" [plain @46:5..46:16]: "Name of newly created SNS Topic" [plain @46:18..46:49]
          "Value" [plain @47:5..47:10]: "SNSTopic.TopicName" [plain @47:20..47:38] !<!GetAtt>
    "#);
}
