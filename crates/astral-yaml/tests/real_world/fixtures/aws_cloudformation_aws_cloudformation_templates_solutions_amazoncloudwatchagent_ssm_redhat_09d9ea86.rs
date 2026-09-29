#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-amazoncloudwatchagent-ssm-redhat-09d9ea86"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Template to install CloudWatchAgent on redhat. It was validated on redhat 7.5" [plain @3:14..3:91]
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
        "RHELVersion" [plain @22:3..22:14]: mapping [block]
          "Type" [plain @23:5..23:9]: "String" [plain @23:11..23:17]
          "AllowedValues" [plain @24:5..24:18]: sequence [block]
            - "RHEL9" [plain @25:9..25:14]
          "Default" [plain @26:5..26:12]: "RHEL9" [plain @26:14..26:19]
          "Description" [plain @27:5..27:16]: "RHEL version to deploy" [plain @27:18..27:40]
        "IAMRole" [plain @29:3..29:10]: mapping [block]
          "Description" [plain @30:5..30:16]: "EC2 attached IAM role" [plain @30:18..30:39]
          "Type" [plain @31:5..31:9]: "String" [plain @31:11..31:17]
          "Default" [plain @32:5..32:12]: "CloudWatchAgentAdminRole" [plain @32:14..32:38]
          "ConstraintDescription" [plain @33:5..33:26]: "must be an existing IAM role which will be attached to EC2 instance." [plain @33:28..33:96]
        "SSHLocation" [plain @35:3..35:14]: mapping [block]
          "Description" [plain @36:5..36:16]: "The IP address range that can be used to SSH to the EC2 instances" [plain @36:18..36:83]
          "Type" [plain @37:5..37:9]: "String" [plain @37:11..37:17]
          "Default" [plain @38:5..38:12]: "0.0.0.0/0" [plain @38:14..38:23]
          "MinLength" [plain @39:5..39:14]: "9" [double-quoted @39:16..39:19]
          "MaxLength" [plain @40:5..40:14]: "18" [double-quoted @40:16..40:20]
          "AllowedPattern" [plain @41:5..41:19]: "(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})/(\\d{1,2})" [plain @41:21..41:73]
          "ConstraintDescription" [plain @42:5..42:26]: "must be a valid IP CIDR range of the form x.x.x.x/x." [plain @42:28..42:80]
        "SubnetId" [plain @44:3..44:11]: mapping [block]
          "Type" [plain @45:5..45:9]: "AWS::EC2::Subnet::Id" [plain @45:11..45:31]
      "Mappings" [plain @47:1..47:9]: mapping [block]
        "RegionMap" [plain @48:3..48:12]: mapping [block]
          "us-east-1" [plain @49:5..49:14]: mapping [block]
            "RHEL9" [plain @50:7..50:12]: "ami-0fb13bb53494158e9" [plain @50:14..50:35]
          "us-east-2" [plain @51:5..51:14]: mapping [block]
            "RHEL9" [plain @52:7..52:12]: "ami-0aeea2f24f6d3ba32" [plain @52:14..52:35]
          "us-west-1" [plain @53:5..53:14]: mapping [block]
            "RHEL9" [plain @54:7..54:12]: "ami-068c2af1200ef7356" [plain @54:14..54:35]
          "us-west-2" [plain @55:5..55:14]: mapping [block]
            "RHEL9" [plain @56:7..56:12]: "ami-0367f2b5c3d1ef960" [plain @56:14..56:35]
          "eu-west-1" [plain @57:5..57:14]: mapping [block]
            "RHEL9" [plain @58:7..58:12]: "ami-0e28d6c0c65e7f82f" [plain @58:14..58:35]
          "eu-west-2" [plain @59:5..59:14]: mapping [block]
            "RHEL9" [plain @60:7..60:12]: "ami-02b1e3a99e36afd1a" [plain @60:14..60:35]
          "ap-northeast-1" [plain @61:5..61:19]: mapping [block]
            "RHEL9" [plain @62:7..62:12]: "ami-0eade93757ef7bb6c" [plain @62:14..62:35]
          "ap-northeast-2" [plain @63:5..63:19]: mapping [block]
            "RHEL9" [plain @64:7..64:12]: "ami-097698b6cd8164ea2" [plain @64:14..64:35]
          "ap-southeast-1" [plain @65:5..65:19]: mapping [block]
            "RHEL9" [plain @66:7..66:12]: "ami-0b9521fddc9871128" [plain @66:14..66:35]
          "ap-southeast-2" [plain @67:5..67:19]: mapping [block]
            "RHEL9" [plain @68:7..68:12]: "ami-0eea634029e7b983c" [plain @68:14..68:35]
      "Resources" [plain @72:1..72:10]: mapping [block]
        "EC2Instance" [plain @73:3..73:14]: mapping [block]
          "CreationPolicy" [plain @74:5..74:19]: mapping [block]
            "ResourceSignal" [plain @75:7..75:21]: mapping [block]
              "Count" [plain @76:9..76:14]: "1" [plain @76:16..76:17]
              "Timeout" [plain @77:9..77:16]: "PT15M" [plain @77:18..77:23]
          "Type" [plain @78:5..78:9]: "AWS::EC2::Instance" [plain @78:11..78:29]
          "Metadata" [plain @79:5..79:13]: mapping [block]
            "AWS::CloudFormation::Init" [plain @80:7..80:32]: mapping [block]
              "configSets" [plain @81:9..81:19]: mapping [block]
                "default" [plain @82:11..82:18]: sequence [block]
                  - "01_setupCfnHup" [plain @83:15..83:29]
                  - "02_config-amazon-cloudwatch-agent" [plain @84:15..84:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @85:15..85:49]
                "UpdateEnvironment" [plain @86:11..86:28]: sequence [block]
                  - "02_config-amazon-cloudwatch-agent" [plain @87:15..87:48]
                  - "03_restart_amazon-cloudwatch-agent" [plain @88:15..88:49]
              "02_config-amazon-cloudwatch-agent" [plain @91:9..91:42]: mapping [block]
                "files" [plain @92:11..92:16]: mapping [block]
                  "/opt/aws/amazon-cloudwatch-agent/etc/dummy.version" [plain @93:13..93:63]: mapping [block]
                    "content" [plain @94:15..94:22]: literal [@94:24..98:1]
                      "\"You can change the VERSION below to to simulate the update of metadata\"\n"
                      "\"VERSION=1.0\"\n"
              "03_restart_amazon-cloudwatch-agent" [plain @99:9..99:43]: mapping [block]
                "commands" [plain @100:11..100:19]: mapping [block]
                  "01_stop_service" [plain @101:13..101:28]: mapping [block]
                    "command" [plain @102:15..102:22]: "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a stop" [plain @102:24..102:96]
                  "02_start_service" [plain @103:13..103:29]: mapping [block]
                    "command" [plain @104:15..104:22]: sequence [block] !<!Sub>
                      - literal [@105:19..107:1]
                        "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a fetch-config -m ec2 -c ssm:${ssmkey} -s\n"
                      - mapping [block]
                        "ssmkey" [plain @107:19..107:25]: "SSMKey" [plain @107:32..107:38] !<!Ref>
              "01_setupCfnHup" [plain @111:9..111:23]: mapping [block]
                "files" [plain @112:11..112:16]: mapping [block]
                  "/etc/cfn/cfn-hup.conf" [plain @113:13..113:34]: mapping [block]
                    "content" [plain @114:15..114:22]: literal [@114:29..119:1] !<!Sub>
                      "[main]\n"
                      "stack=${AWS::StackId}\n"
                      "region=${AWS::Region}\n"
                      "interval=1\n"
                    "mode" [plain @119:15..119:19]: "000400" [double-quoted @119:21..119:29]
                    "owner" [plain @120:15..120:20]: "root" [plain @120:22..120:26]
                    "group" [plain @121:15..121:20]: "root" [plain @121:22..121:26]
                  "/etc/cfn/hooks.d/amazon-cloudwatch-agent-auto-reloader.conf" [plain @122:13..122:72]: mapping [block]
                    "content" [plain @123:15..123:22]: literal [@123:29..129:1] !<!Sub>
                      "[cfn-auto-reloader-hook]\n"
                      "triggers=post.update\n"
                      "path=Resources.EC2Instance.Metadata.AWS::CloudFormation::Init.02_config-amazon-cloudwatch-agent\n"
                      "action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets UpdateEnvironment\n"
                      "runas=root\n"
                    "mode" [plain @129:15..129:19]: "000400" [double-quoted @129:21..129:29]
                    "owner" [plain @130:15..130:20]: "root" [plain @130:22..130:26]
                    "group" [plain @131:15..131:20]: "root" [plain @131:22..131:26]
                  "/lib/systemd/system/cfn-hup.service" [plain @132:13..132:48]: mapping [block]
                    "content" [plain @133:15..133:22]: literal [@133:24..142:1]
                      "[Unit]\n"
                      "Description=cfn-hup daemon\n"
                      "[Service]\n"
                      "Type=simple\n"
                      "ExecStart=/opt/aws/bin/cfn-hup\n"
                      "Restart=always\n"
                      "[Install]\n"
                      "WantedBy=multi-user.target\n"
                "commands" [plain @142:11..142:19]: mapping [block]
                  "01enable_cfn_hup" [plain @143:13..143:29]: mapping [block]
                    "command" [plain @144:15..144:22]: "systemctl enable cfn-hup.service" [plain @144:24..144:56]
                  "02start_cfn_hup" [plain @145:13..145:28]: mapping [block]
                    "command" [plain @146:15..146:22]: "systemctl start cfn-hup.service" [plain @146:24..146:55]
          "Properties" [plain @147:5..147:15]: mapping [block]
            "InstanceType" [plain @148:7..148:19]: "InstanceType" [plain @148:26..148:38] !<!Ref>
            "IamInstanceProfile" [plain @149:7..149:25]: "IAMRole" [plain @149:32..149:39] !<!Ref>
            "KeyName" [plain @150:7..150:14]: "KeyName" [plain @150:21..150:28] !<!Ref>
            "ImageId" [plain @151:7..151:14]: sequence [block] !<!FindInMap>
              - "RegionMap" [plain @152:11..152:20]
              - "AWS::Region" [single-quoted @153:16..153:29] !<!Ref>
              - "RHELVersion" [plain @154:16..154:27] !<!Ref>
            "SubnetId" [plain @155:7..155:15]: "SubnetId" [plain @155:22..155:30] !<!Ref>
            "SecurityGroupIds" [plain @156:7..156:23]: sequence [block]
              - "InstanceSecurityGroup.GroupId" [plain @157:19..157:48] !<!GetAtt>
            "UserData" [plain @158:7..158:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @159:9..159:16]: sequence [block]
                - literal [@160:13..173:1]
                  "#!/bin/bash\n"
                  "rpm -Uvh https://s3.amazonaws.com/amazoncloudwatch-agent/redhat/amd64/latest/amazon-cloudwatch-agent.rpm\n"
                  "/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a fetch-config -m ec2 -c ssm:${ssmkey} -s\n"
                  "yum update -y\n"
                  "yum install python3 -y\n"
                  "curl -O https://bootstrap.pypa.io/get-pip.py\n"
                  "# Install pip using python3\n"
                  "python3 get-pip.py\n"
                  "export PATH=$PATH:/usr/local/bin\n"
                  "pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz\n"
                  "cfn-init -v --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region} --configsets default\n"
                  "cfn-signal -e $? --stack ${AWS::StackId} --resource EC2Instance --region ${AWS::Region}\n"
                - mapping [block]
                  "ssmkey" [plain @173:13..173:19]: "SSMKey" [plain @173:26..173:32] !<!Ref>
        "InstanceSecurityGroup" [plain @175:3..175:24]: mapping [block]
          "Type" [plain @176:5..176:9]: "AWS::EC2::SecurityGroup" [plain @176:11..176:34]
          "Properties" [plain @177:5..177:15]: mapping [block]
            "GroupDescription" [plain @178:7..178:23]: "Enable SSH access via port 22" [plain @178:25..178:54]
            "SecurityGroupIngress" [plain @179:7..179:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @180:11..180:21]: "tcp" [plain @180:23..180:26]
                "FromPort" [plain @181:11..181:19]: "22" [double-quoted @181:21..181:25]
                "ToPort" [plain @182:11..182:17]: "22" [double-quoted @182:19..182:23]
                "CidrIp" [plain @183:11..183:17]: "SSHLocation" [plain @183:24..183:35] !<!Ref>
    "##);
}
