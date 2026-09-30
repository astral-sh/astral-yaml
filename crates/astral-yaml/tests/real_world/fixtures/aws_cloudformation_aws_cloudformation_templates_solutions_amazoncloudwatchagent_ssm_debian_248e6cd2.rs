#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-amazoncloudwatchagent-ssm-debian-248e6cd2"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Template to install CloudWatchAgent on debian. It was validated on debian 12.0" [plain @3:14..3:92]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "SSMKey" [plain @6:3..6:9]: mapping [block]
          "Description" [plain @7:5..7:16]: "Name of parameter store which contains the json configuration of CWAgent." [plain @7:18..7:91]
          "Type" [plain @8:5..8:9]: "String" [plain @8:11..8:17]
          "Default" [plain @9:5..9:12]: "AmazonCloudWatch-DefaultLinuxConfigCloudFormation" [plain @9:14..9:63]
        "KeyName" [plain @11:3..11:10]: mapping [block]
          "Description" [plain @12:5..12:16]: "Name of an existing EC2 KeyPair to enable SSH access to the instance" [plain @12:18..12:86]
          "Type" [plain @13:5..13:9]: "AWS::EC2::KeyPair::KeyName" [plain @13:11..13:37]
          "ConstraintDescription" [plain @14:5..14:26]: "must be the name of an existing EC2 KeyPair." [plain @14:28..14:72]
        "InstanceType" [plain @16:3..16:15]: mapping [block]
          "Description" [plain @17:5..17:16]: "EC2 instance type" [plain @17:18..17:35]
          "Type" [plain @18:5..18:9]: "String" [plain @18:11..18:17]
          "Default" [plain @19:5..19:12]: "t3.medium" [plain @19:14..19:23]
          "ConstraintDescription" [plain @20:5..20:26]: "must be a valid EC2 instance type." [plain @20:28..20:62]
        "InstanceAMI" [plain @22:3..22:14]: mapping [block]
          "Description" [plain @23:5..23:16]: "Managed AMI ID for EC2 Instance" [plain @23:18..23:49]
          "Type" [plain @24:5..24:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @24:11..24:58]
          "Default" [plain @25:5..25:12]: "/aws/service/debian/release/10/latest/amd64" [plain @25:14..25:57]
        "IAMRole" [plain @27:3..27:10]: mapping [block]
          "Description" [plain @28:5..28:16]: "EC2 attached IAM role" [plain @28:18..28:39]
          "Type" [plain @29:5..29:9]: "String" [plain @29:11..29:17]
          "Default" [plain @30:5..30:12]: "CloudWatchAgentAdminRole" [plain @30:14..30:38]
          "ConstraintDescription" [plain @31:5..31:26]: "must be an existing IAM role which will be attached to EC2 instance." [plain @31:28..31:96]
        "SSHLocation" [plain @33:3..33:14]: mapping [block]
          "Description" [plain @34:5..34:16]: "The IP address range that can be used to SSH to the EC2 instances" [plain @34:18..34:83]
          "Type" [plain @35:5..35:9]: "String" [plain @35:11..35:17]
          "Default" [plain @36:5..36:12]: "0.0.0.0/0" [plain @36:14..36:23]
          "MinLength" [plain @37:5..37:14]: "9" [double-quoted @37:16..37:19]
          "MaxLength" [plain @38:5..38:14]: "18" [double-quoted @38:16..38:20]
          "AllowedPattern" [plain @39:5..39:19]: "(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})/(\\d{1,2})" [plain @39:21..39:73]
          "ConstraintDescription" [plain @40:5..40:26]: "must be a valid IP CIDR range of the form x.x.x.x/x." [plain @40:28..40:80]
        "SubnetId" [plain @42:3..42:11]: mapping [block]
          "Type" [plain @43:5..43:9]: "AWS::EC2::Subnet::Id" [plain @43:11..43:31]
      "Resources" [plain @45:1..45:10]: mapping [block]
        "EC2Instance" [plain @46:3..46:14]: mapping [block]
          "CreationPolicy" [plain @47:5..47:19]: mapping [block]
            "ResourceSignal" [plain @48:7..48:21]: mapping [block]
              "Count" [plain @49:9..49:14]: "1" [plain @49:16..49:17]
              "Timeout" [plain @50:9..50:16]: "PT15M" [plain @50:18..50:23]
          "Type" [plain @51:5..51:9]: "AWS::EC2::Instance" [plain @51:11..51:29]
          "Metadata" [plain @52:5..52:13]: mapping [block]
            "AWS::CloudFormation::Init" [plain @53:7..53:32]: mapping [block]
              "configSets" [plain @54:9..54:19]: mapping [block]
                "default" [plain @55:11..55:18]: sequence [block]
                  - "01_setupCfnHup" [plain @56:15..56:29]
                  - "02_config-amazon-cloudwatch-agent" [plain @57:15..57:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @58:15..58:49]
                "UpdateEnvironment" [plain @59:11..59:28]: sequence [block]
                  - "02_config-amazon-cloudwatch-agent" [plain @60:15..60:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @61:15..61:49]
              "02_config-amazon-cloudwatch-agent" [plain @64:9..64:42]: mapping [block]
                "files" [plain @65:11..65:16]: mapping [block]
                  "/opt/aws/amazon-cloudwatch-agent/etc/dummy.version" [plain @66:13..66:63]: mapping [block]
                    "content" [plain @67:15..67:22]: literal [@67:24..71:1]
                      "\"You can change the VERSION below to to simulate the update of metadata\"\n"
                      "\"VERSION=1.0\"\n"
              "03_restart_amazon-cloudwatch-agent" [plain @72:9..72:43]: mapping [block]
                "commands" [plain @73:11..73:19]: mapping [block]
                  "01_stop_service" [plain @74:13..74:28]: mapping [block]
                    "command" [plain @75:15..75:22]: "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a stop" [plain @75:24..75:96]
                  "02_start_service" [plain @76:13..76:29]: mapping [block]
                    "command" [plain @77:15..77:22]: sequence [block] !<!Sub>
                      - literal [@78:19..80:1]
                        "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a fetch-config -m ec2 -c ssm:${ssmkey} -s\n"
                      - mapping [block]
                        "ssmkey" [plain @80:19..80:25]: "SSMKey" [plain @80:32..80:38] !<!Ref>
              "01_setupCfnHup" [plain @84:9..84:23]: mapping [block]
                "files" [plain @85:11..85:16]: mapping [block]
                  "/etc/cfn/cfn-hup.conf" [plain @86:13..86:34]: mapping [block]
                    "content" [plain @87:15..87:22]: literal [@87:29..92:1] !<!Sub>
                      "[main]\n"
                      "stack=${AWS::StackId}\n"
                      "region=${AWS::Region}\n"
                      "interval=1\n"
                    "mode" [plain @92:15..92:19]: "000400" [double-quoted @92:21..92:29]
                    "owner" [plain @93:15..93:20]: "root" [plain @93:22..93:26]
                    "group" [plain @94:15..94:20]: "root" [plain @94:22..94:26]
                  "/etc/cfn/hooks.d/amazon-cloudwatch-agent-auto-reloader.conf" [plain @95:13..95:72]: mapping [block]
                    "content" [plain @96:15..96:22]: literal [@96:29..102:1] !<!Sub>
                      "[cfn-auto-reloader-hook]\n"
                      "triggers=post.update\n"
                      "path=Resources.EC2Instance.Metadata.AWS::CloudFormation::Init.02_config-amazon-cloudwatch-agent\n"
                      "action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets UpdateEnvironment\n"
                      "runas=root\n"
                    "mode" [plain @102:15..102:19]: "000400" [double-quoted @102:21..102:29]
                    "owner" [plain @103:15..103:20]: "root" [plain @103:22..103:26]
                    "group" [plain @104:15..104:20]: "root" [plain @104:22..104:26]
                  "/lib/systemd/system/cfn-hup.service" [plain @105:13..105:48]: mapping [block]
                    "content" [plain @106:15..106:22]: literal [@106:24..115:1]
                      "[Unit]\n"
                      "Description=cfn-hup daemon\n"
                      "[Service]\n"
                      "Type=simple\n"
                      "ExecStart=/opt/aws/bin/cfn-hup\n"
                      "Restart=always\n"
                      "[Install]\n"
                      "WantedBy=multi-user.target\n"
                "commands" [plain @115:11..115:19]: mapping [block]
                  "01enable_cfn_hup" [plain @116:13..116:29]: mapping [block]
                    "command" [plain @117:15..117:22]: "systemctl enable cfn-hup.service" [plain @117:24..117:56]
                  "02start_cfn_hup" [plain @118:13..118:28]: mapping [block]
                    "command" [plain @119:15..119:22]: "systemctl start cfn-hup.service" [plain @119:24..119:55]
          "Properties" [plain @120:5..120:15]: mapping [block]
            "InstanceType" [plain @121:7..121:19]: "InstanceType" [plain @121:26..121:38] !<!Ref>
            "IamInstanceProfile" [plain @122:7..122:25]: "IAMRole" [plain @122:32..122:39] !<!Ref>
            "KeyName" [plain @123:7..123:14]: "KeyName" [plain @123:21..123:28] !<!Ref>
            "ImageId" [plain @124:7..124:14]: "InstanceAMI" [plain @124:21..124:32] !<!Ref>
            "SubnetId" [plain @125:7..125:15]: "SubnetId" [plain @125:22..125:30] !<!Ref>
            "SecurityGroupIds" [plain @126:7..126:23]: sequence [block]
              - "InstanceSecurityGroup.GroupId" [plain @127:19..127:48] !<!GetAtt>
            "UserData" [plain @128:7..128:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @129:9..129:16]: sequence [block]
                - literal [@130:13..153:1]
                  "#!/bin/bash\n"
                  "wget https://s3.amazonaws.com/amazoncloudwatch-agent/debian/amd64/latest/amazon-cloudwatch-agent.deb -O /tmp/amazon-cloudwatch-agent.deb\n"
                  "sudo dpkg -i /tmp/amazon-cloudwatch-agent.deb\n"
                  "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a fetch-config -m ec2 -c ssm:${ssmkey} -s\n"
                  "wget https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz -O /tmp/aws-cfn-bootstrap-py3-latest.tar.gz\n"
                  "sudo apt-get update -y\n"
                  "sudo apt-get install -y python3-pip python3-venv\n"
                  "\n"
                  "# Create and activate a virtual environment\n"
                  "python3 -m venv /opt/aws/virtualenv\n"
                  "source /opt/aws/virtualenv/bin/activate\n"
                  "\n"
                  "# Install the bootstrap package\n"
                  "pip install /tmp/aws-cfn-bootstrap-py3-latest.tar.gz\n"
                  "\n"
                  "# Create necessary symlinks\n"
                  "sudo mkdir -p /opt/aws/bin\n"
                  "sudo ln -s /opt/aws/virtualenv/bin/cfn-* /opt/aws/bin/\n"
                  "\n"
                  "# Run cfn-init\n"
                  "/opt/aws/bin/cfn-init -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets default\n"
                  "/opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region}\n"
                - mapping [block]
                  "ssmkey" [plain @153:13..153:19]: "SSMKey" [plain @153:26..153:32] !<!Ref>
        "InstanceSecurityGroup" [plain @155:3..155:24]: mapping [block]
          "Type" [plain @156:5..156:9]: "AWS::EC2::SecurityGroup" [plain @156:11..156:34]
          "Properties" [plain @157:5..157:15]: mapping [block]
            "GroupDescription" [plain @158:7..158:23]: "Enable SSH access via port 22" [plain @158:25..158:54]
            "SecurityGroupIngress" [plain @159:7..159:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @160:11..160:21]: "tcp" [plain @160:23..160:26]
                "FromPort" [plain @161:11..161:19]: "22" [double-quoted @161:21..161:25]
                "ToPort" [plain @162:11..162:17]: "22" [double-quoted @162:19..162:23]
                "CidrIp" [plain @163:11..163:17]: "SSHLocation" [plain @163:24..163:35] !<!Ref>
    "##);
}
