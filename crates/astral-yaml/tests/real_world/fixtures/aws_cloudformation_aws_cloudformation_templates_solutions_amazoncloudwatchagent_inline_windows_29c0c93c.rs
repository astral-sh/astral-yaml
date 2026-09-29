#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-amazoncloudwatchagent-inline-windows-29c0c93c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Template to install CloudWatchAgent on windows. It was validated on windows 2016" [plain @3:14..3:94]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "KeyName" [plain @6:3..6:10]: mapping [block]
          "Description" [plain @7:5..7:16]: "Name of an existing EC2 KeyPair to enable SSH access to the instance" [plain @7:18..7:86]
          "Type" [plain @8:5..8:9]: "AWS::EC2::KeyPair::KeyName" [plain @8:11..8:37]
          "ConstraintDescription" [plain @9:5..9:26]: "must be the name of an existing EC2 KeyPair." [plain @9:28..9:72]
        "InstanceType" [plain @11:3..11:15]: mapping [block]
          "Description" [plain @12:5..12:16]: "EC2 instance type" [plain @12:18..12:35]
          "Type" [plain @13:5..13:9]: "String" [plain @13:11..13:17]
          "Default" [plain @14:5..14:12]: "t3.medium" [plain @14:14..14:23]
          "ConstraintDescription" [plain @15:5..15:26]: "must be a valid EC2 instance type." [plain @15:28..15:62]
        "InstanceAMI" [plain @17:3..17:14]: mapping [block]
          "Description" [plain @18:5..18:16]: "Managed AMI ID for EC2 Instance" [plain @18:18..18:49]
          "Type" [plain @19:5..19:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @19:11..19:58]
          "Default" [plain @20:5..20:12]: "/aws/service/ami-windows-latest/Windows_Server-2022-English-Full-SQL_2022_Web" [plain @20:14..20:91]
        "IAMRole" [plain @22:3..22:10]: mapping [block]
          "Description" [plain @23:5..23:16]: "EC2 attached IAM role" [plain @23:18..23:39]
          "Type" [plain @24:5..24:9]: "String" [plain @24:11..24:17]
          "Default" [plain @25:5..25:12]: "CloudWatchAgentAdminRole" [plain @25:14..25:38]
          "ConstraintDescription" [plain @26:5..26:26]: "must be an existing IAM role which will be attached to EC2 instance." [plain @26:28..26:96]
        "SubnetId" [plain @28:3..28:11]: mapping [block]
          "Type" [plain @29:5..29:9]: "AWS::EC2::Subnet::Id" [plain @29:11..29:31]
      "Resources" [plain @31:1..31:10]: mapping [block]
        "EC2Instance" [plain @32:3..32:14]: mapping [block]
          "CreationPolicy" [plain @33:5..33:19]: mapping [block]
            "ResourceSignal" [plain @34:7..34:21]: mapping [block]
              "Count" [plain @35:9..35:14]: "1" [plain @35:16..35:17]
              "Timeout" [plain @36:9..36:16]: "PT15M" [plain @36:18..36:23]
          "Type" [plain @37:5..37:9]: "AWS::EC2::Instance" [plain @37:11..37:29]
          "Metadata" [plain @38:5..38:13]: mapping [block]
            "AWS::CloudFormation::Init" [plain @39:7..39:32]: mapping [block]
              "configSets" [plain @40:9..40:19]: mapping [block]
                "default" [plain @41:11..41:18]: sequence [block]
                  - "00_setupCfnHup" [plain @42:15..42:29]
                  - "01_CfnHup_service" [plain @43:15..43:32]
                  - "02_config-amazon-cloudwatch-agent" [plain @44:15..44:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @45:15..45:49]
                "UpdateEnvironment" [plain @46:11..46:28]: sequence [block]
                  - "02_config-amazon-cloudwatch-agent" [plain @47:15..47:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @48:15..48:49]
              "00_setupCfnHup" [plain @52:9..52:23]: mapping [block]
                "files" [plain @53:11..53:16]: mapping [block]
                  "c:\\cfn\\cfn-hup.conf" [plain @54:13..54:32]: mapping [block]
                    "content" [plain @55:15..55:22]: literal [@55:29..60:1] !<!Sub>
                      "[main]\n"
                      "stack=${AWS::StackId}\n"
                      "region=${AWS::Region}\n"
                      "interval=1\n"
                  "c:\\cfn\\hooks.d\\amazon-cloudwatch-agent-auto-reloader.conf" [plain @60:13..60:70]: mapping [block]
                    "content" [plain @61:15..61:22]: literal [@61:29..66:1] !<!Sub>
                      "[cfn-auto-reloader-hook]\n"
                      "triggers=post.update\n"
                      "path=Resources.EC2Instance.Metadata.AWS::CloudFormation::Init.02_config-amazon-cloudwatch-agent\n"
                      "action=cfn-init.exe -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets UpdateEnvironment\n"
              "01_CfnHup_service" [plain @66:9..66:26]: mapping [block]
                "services" [plain @67:11..67:19]: mapping [block]
                  "windows" [plain @68:13..68:20]: mapping [block]
                    "cfn-hup" [plain @69:15..69:22]: mapping [block]
                      "enabled" [plain @70:17..70:24]: "true" [double-quoted @70:26..70:32]
                      "ensureRunning" [plain @71:17..71:30]: "true" [double-quoted @71:32..71:38]
                      "files" [plain @72:17..72:22]: sequence [block]
                        - "c:\\cfn\\cfn-hup.conf" [plain @73:21..73:40]
                        - "c:\\cfn\\hooks.d\\cfn-auto-reloader.conf" [plain @74:21..74:58]
              "02_config-amazon-cloudwatch-agent" [plain @77:9..77:42]: mapping [block]
                "files" [plain @78:11..78:16]: mapping [block]
                  "C:\\ProgramData\\Amazon\\AmazonCloudWatchAgent\\amazon-cloudwatch-agent.json" [plain @79:13..79:85]: mapping [block]
                    "content" [plain @80:15..80:22]: literal [@80:24..109:1]
                      "{\n"
                      "  \"metrics\": {\n"
                      "    \"append_dimensions\": {\n"
                      "      \"AutoScalingGroupName\": \"${!aws:AutoScalingGroupName}\",\n"
                      "      \"ImageId\": \"${!aws:ImageId}\",\n"
                      "      \"InstanceId\": \"${!aws:InstanceId}\",\n"
                      "      \"InstanceType\": \"${!aws:InstanceType}\"\n"
                      "    },\n"
                      "    \"metrics_collected\": {\n"
                      "      \"Memory\": {\n"
                      "        \"measurement\": [\n"
                      "          \"% Committed Bytes In Use\"\n"
                      "        ],\n"
                      "        \"metrics_collection_interval\": 60\n"
                      "      },\n"
                      "      \"Paging File\": {\n"
                      "        \"measurement\": [\n"
                      "          \"% Usage\"\n"
                      "        ],\n"
                      "        \"metrics_collection_interval\": 60,\n"
                      "        \"resources\": [\n"
                      "          \"*\"\n"
                      "        ]\n"
                      "      }\n"
                      "    }\n"
                      "  }\n"
                      "}\n"
              "03_restart_amazon-cloudwatch-agent" [plain @110:9..110:43]: mapping [block]
                "commands" [plain @111:11..111:19]: mapping [block]
                  "01_stop_service" [plain @112:13..112:28]: mapping [block]
                    "command" [plain @113:15..113:22]: "powershell -Command \"C:\\\\'Program Files'\\\\Amazon\\\\AmazonCloudWatchAgent\\\\amazon-cloudwatch-agent-ctl.ps1 -a stop\"" [plain @113:24..113:137]
                  "02_start_service" [plain @114:13..114:29]: mapping [block]
                    "command" [plain @115:15..115:22]: "powershell -Command \"C:\\\\'Program Files'\\\\Amazon\\\\AmazonCloudWatchAgent\\\\amazon-cloudwatch-agent-ctl.ps1 -a fetch-config -m ec2 -c file:C:\\\\ProgramData\\\\Amazon\\\\AmazonCloudWatchAgent\\\\amazon-cloudwatch-agent.json -s\"" [plain @115:24..115:240]
          "Properties" [plain @116:5..116:15]: mapping [block]
            "InstanceType" [plain @117:7..117:19]: "InstanceType" [plain @117:26..117:38] !<!Ref>
            "IamInstanceProfile" [plain @118:7..118:25]: "IAMRole" [plain @118:32..118:39] !<!Ref>
            "KeyName" [plain @119:7..119:14]: "KeyName" [plain @119:21..119:28] !<!Ref>
            "ImageId" [plain @120:7..120:14]: "InstanceAMI" [plain @120:21..120:32] !<!Ref>
            "SubnetId" [plain @121:7..121:15]: "SubnetId" [plain @121:22..121:30] !<!Ref>
            "UserData" [plain @122:7..122:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @123:9..123:16]: literal [@123:18..131:1]
                "<script>\n"
                "mkdir C:\\Downloads\\Amazon\\AmazonCloudWatchAgent\n"
                "powershell -Command \"(New-Object Net.WebClient).DownloadFile('https://s3.amazonaws.com/amazoncloudwatch-agent/windows/amd64/latest/amazon-cloudwatch-agent.msi','C:\\Downloads\\Amazon\\AmazonCloudWatchAgent\\amazon-cloudwatch-agent.msi')\"\n"
                "C:\\Downloads\\Amazon\\AmazonCloudWatchAgent\\amazon-cloudwatch-agent.msi\n"
                "cfn-init.exe -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets default\n"
                "cfn-signal.exe -e %errorlevel% --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region}\n"
                "</script>"
    "#);
}
