#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-sqs-sqsstandardqueue-2a04457c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Best Practice SQS Standard Queue" [plain @3:14..3:46]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "DelaySeconds" [plain @6:3..6:15]: mapping [block]
          "Description" [plain @7:5..7:16]: "The time in seconds that the delivery of all messages in the queue is delayed. You can specify an integer value of 0 to 900 (15 minutes)." [plain @7:18..7:155]
          "Type" [plain @8:5..8:9]: "Number" [plain @8:11..8:17]
          "Default" [plain @9:5..9:12]: "5" [double-quoted @9:14..9:17]
        "MaximumMessageSize" [plain @11:3..11:21]: mapping [block]
          "Description" [plain @12:5..12:16]: "The limit of how many bytes that a message can contain before Amazon SQS rejects it, 1024 bytes (1 KiB) to 262144 bytes (256 KiB)" [plain @12:18..12:147]
          "Type" [plain @13:5..13:9]: "Number" [plain @13:11..13:17]
          "Default" [plain @14:5..14:12]: "262144" [double-quoted @14:14..14:22]
        "MessageRetentionPeriod" [plain @16:3..16:25]: mapping [block]
          "Description" [plain @17:5..17:16]: "The number of seconds that Amazon SQS retains a message. You can specify an integer value from 60 seconds (1 minute) to 1209600 seconds (14 days). " [single-quoted @17:18..17:167]
          "Type" [plain @18:5..18:9]: "Number" [plain @18:11..18:17]
          "Default" [plain @19:5..19:12]: "345600" [double-quoted @19:14..19:22]
        "ReceiveMessageWaitTimeSeconds" [plain @21:3..21:32]: mapping [block]
          "Description" [plain @22:5..22:16]: "Specifies the duration, in seconds, that the ReceiveMessage action call waits until a message is in the queue in order to include it in the response, as opposed to returning an empty response if a message is not yet available. 1 to 20" [plain @22:18..22:252]
          "Type" [plain @23:5..23:9]: "Number" [plain @23:11..23:17]
          "Default" [plain @24:5..24:12]: "0" [double-quoted @24:14..24:17]
        "UsedeadletterQueue" [plain @26:3..26:21]: mapping [block]
          "Description" [plain @27:5..27:16]: "A dead-letter queue is a queue that other (source) queues can target for messages that can't be processed (consumed) successfully. You can set aside and isolate these messages in the dead-letter queue to determine why their processing doesn't succeed." [plain @27:18..27:269]
          "Type" [plain @28:5..28:9]: "String" [plain @28:11..28:17]
          "AllowedValues" [plain @29:5..29:18]: sequence [block]
            - "true" [double-quoted @30:9..30:15]
            - "false" [double-quoted @31:9..31:16]
          "Default" [plain @32:5..32:12]: "false" [double-quoted @32:14..32:21]
        "VisibilityTimeout" [plain @34:3..34:20]: mapping [block]
          "Description" [plain @35:5..35:16]: "Time in seconds. This should be longer than the time it would take to process and delete a message, this should not exceed 12 hours." [plain @35:18..35:150]
          "Type" [plain @36:5..36:9]: "Number" [plain @36:11..36:17]
          "Default" [plain @37:5..37:12]: "5" [double-quoted @37:14..37:17]
        "KmsMasterKeyIdForSqs" [plain @39:3..39:23]: mapping [block]
          "Description" [plain @40:5..40:16]: "(Optional) For unencrypted leave blank. The ID or Alias of an AWS managed or a custom CMK." [plain @40:18..40:108]
          "Type" [plain @41:5..41:9]: "String" [plain @41:11..41:17]
          "Default" [plain @42:5..42:12]: "alias/aws/sqs" [plain @42:14..42:27]
      "Mappings" [plain @44:1..44:9]: mapping [flow]
      "Conditions" [plain @46:1..46:11]: mapping [block]
        "CreateDeadLetterQueue" [plain @47:3..47:24]: sequence [block] !<!Equals>
          - "UsedeadletterQueue" [plain @48:12..48:30] !<!Ref>
          - "true" [double-quoted @49:7..49:13]
        "IsKmsExist" [plain @51:3..51:13]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "" [double-quoted @53:9..53:11]
            - "KmsMasterKeyIdForSqs" [plain @54:14..54:34] !<!Ref>
      "Resources" [plain @56:1..56:10]: mapping [block]
        "SQSQueue" [plain @57:3..57:11]: mapping [block]
          "Type" [plain @58:5..58:9]: "AWS::SQS::Queue" [plain @58:11..58:26]
          "Properties" [plain @59:5..59:15]: mapping [block]
            "DelaySeconds" [plain @60:7..60:19]: "DelaySeconds" [plain @60:26..60:38] !<!Ref>
            "MaximumMessageSize" [plain @61:7..61:25]: "MaximumMessageSize" [plain @61:32..61:50] !<!Ref>
            "MessageRetentionPeriod" [plain @62:7..62:29]: "MessageRetentionPeriod" [plain @62:36..62:58] !<!Ref>
            "ReceiveMessageWaitTimeSeconds" [plain @63:7..63:36]: "ReceiveMessageWaitTimeSeconds" [plain @63:43..63:72] !<!Ref>
            "KmsMasterKeyId" [plain @64:7..64:21]: sequence [block] !<!If>
              - "IsKmsExist" [plain @65:11..65:21]
              - "KmsMasterKeyIdForSqs" [plain @66:16..66:36] !<!Ref>
              - "AWS::NoValue" [plain @67:16..67:28] !<!Ref>
            "RedrivePolicy" [plain @68:7..68:20]: sequence [block] !<!If>
              - "CreateDeadLetterQueue" [plain @69:11..69:32]
              - mapping [block]
                "deadLetterTargetArn" [plain @70:11..70:30]: "MyDeadLetterQueue.Arn" [plain @70:40..70:61] !<!GetAtt>
                "maxReceiveCount" [plain @71:11..71:26]: "5" [plain @71:28..71:29]
              - "AWS::NoValue" [plain @72:16..72:28] !<!Ref>
            "VisibilityTimeout" [plain @73:7..73:24]: "VisibilityTimeout" [plain @73:31..73:48] !<!Ref>
        "MyDeadLetterQueue" [plain @75:3..75:20]: mapping [block]
          "Type" [plain @76:5..76:9]: "AWS::SQS::Queue" [plain @76:11..76:26]
          "Properties" [plain @77:5..77:15]: mapping [block]
            "MessageRetentionPeriod" [plain @78:7..78:29]: "1209600" [plain @78:31..78:38]
          "Condition" [plain @79:5..79:14]: "CreateDeadLetterQueue" [plain @79:16..79:37]
      "Outputs" [plain @81:1..81:8]: mapping [block]
        "QueueURL" [plain @82:3..82:11]: mapping [block]
          "Description" [plain @83:5..83:16]: "URL of newly created SQS Queue" [plain @83:18..83:48]
          "Value" [plain @84:5..84:10]: "SQSQueue" [plain @84:17..84:25] !<!Ref>
        "QueueARN" [plain @86:3..86:11]: mapping [block]
          "Description" [plain @87:5..87:16]: "ARN of newly created SQS Queue" [plain @87:18..87:48]
          "Value" [plain @88:5..88:10]: "SQSQueue.Arn" [plain @88:20..88:32] !<!GetAtt>
        "QueueName" [plain @90:3..90:12]: mapping [block]
          "Description" [plain @91:5..91:16]: "Name newly created SQS Queue" [plain @91:18..91:46]
          "Value" [plain @92:5..92:10]: "SQSQueue.QueueName" [plain @92:20..92:38] !<!GetAtt>
        "DeadLetterQueueURL" [plain @94:3..94:21]: mapping [block]
          "Description" [plain @95:5..95:16]: "URL of the dead letter queue" [plain @95:18..95:46]
          "Value" [plain @96:5..96:10]: "MyDeadLetterQueue" [plain @96:17..96:34] !<!Ref>
          "Condition" [plain @97:5..97:14]: "CreateDeadLetterQueue" [plain @97:16..97:37]
        "DeadLetterQueueARN" [plain @99:3..99:21]: mapping [block]
          "Description" [plain @100:5..100:16]: "ARN of the dead letter queue" [plain @100:18..100:46]
          "Value" [plain @101:5..101:10]: "MyDeadLetterQueue.Arn" [plain @101:20..101:41] !<!GetAtt>
          "Condition" [plain @102:5..102:14]: "CreateDeadLetterQueue" [plain @102:16..102:37]
    "#);
}
