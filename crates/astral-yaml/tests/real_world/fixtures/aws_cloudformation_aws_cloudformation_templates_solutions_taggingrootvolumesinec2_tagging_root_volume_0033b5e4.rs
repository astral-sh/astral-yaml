#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-taggingrootvolumesinec2-tagging-root-volume-0033b5e4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "AWS CloudFormation Sample Template Tagging Root Volumes of EC2 Instances: This template shows how to automatically tag the root volume of the EC2 instances which are created through the CloudFormation template. This is done through the UserData property of the AWS::EC2::Instance resource. **WARNING** This template creates two Amazon EC2 instances and an IAM Role. You will be billed for the AWS resources used if you create a stack from this template." [single-quoted @3:14..3:469]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "KeyName" [plain @6:3..6:10]: mapping [block]
          "Description" [plain @7:5..7:16]: "Name of an existing EC2 KeyPair to enable SSH access to the ECS instances." [plain @7:18..7:92]
          "Type" [plain @8:5..8:9]: "AWS::EC2::KeyPair::KeyName" [plain @8:11..8:37]
        "InstanceType" [plain @10:3..10:15]: mapping [block]
          "Description" [plain @11:5..11:16]: "EC2 instance type" [plain @11:18..11:35]
          "Type" [plain @12:5..12:9]: "String" [plain @12:11..12:17]
          "AllowedValues" [plain @13:5..13:18]: sequence [block]
            - "t2.micro" [plain @14:9..14:17]
            - "t2.small" [plain @15:9..15:17]
            - "t2.medium" [plain @16:9..16:18]
            - "t2.large" [plain @17:9..17:17]
            - "m3.medium" [plain @18:9..18:18]
            - "m3.large" [plain @19:9..19:17]
            - "m3.xlarge" [plain @20:9..20:18]
            - "m3.2xlarge" [plain @21:9..21:19]
            - "m4.large" [plain @22:9..22:17]
            - "m4.xlarge" [plain @23:9..23:18]
            - "m4.2xlarge" [plain @24:9..24:19]
            - "m4.4xlarge" [plain @25:9..25:19]
            - "m4.10xlarge" [plain @26:9..26:20]
            - "c4.large" [plain @27:9..27:17]
            - "c4.xlarge" [plain @28:9..28:18]
            - "c4.2xlarge" [plain @29:9..29:19]
            - "c4.4xlarge" [plain @30:9..30:19]
            - "c4.8xlarge" [plain @31:9..31:19]
            - "c3.large" [plain @32:9..32:17]
            - "c3.xlarge" [plain @33:9..33:18]
            - "c3.2xlarge" [plain @34:9..34:19]
            - "c3.4xlarge" [plain @35:9..35:19]
            - "c3.8xlarge" [plain @36:9..36:19]
            - "r3.large" [plain @37:9..37:17]
            - "r3.xlarge" [plain @38:9..38:18]
            - "r3.2xlarge" [plain @39:9..39:19]
            - "r3.4xlarge" [plain @40:9..40:19]
            - "r3.8xlarge" [plain @41:9..41:19]
            - "i2.xlarge" [plain @42:9..42:18]
            - "i2.2xlarge" [plain @43:9..43:19]
            - "i2.4xlarge" [plain @44:9..44:19]
            - "i2.8xlarge" [plain @45:9..45:19]
          "Default" [plain @46:5..46:12]: "t2.micro" [plain @46:14..46:22]
          "ConstraintDescription" [plain @47:5..47:26]: "Please choose a valid instance type." [plain @47:28..47:64]
        "InstanceAZ" [plain @49:3..49:13]: mapping [block]
          "Description" [plain @50:5..50:16]: "EC2 AZ." [plain @50:18..50:25]
          "Type" [plain @51:5..51:9]: "AWS::EC2::AvailabilityZone::Name" [plain @51:11..51:43]
          "ConstraintDescription" [plain @52:5..52:26]: "Must be the name of an availabity zone." [plain @52:28..52:67]
        "WindowsAMIID" [plain @54:3..54:15]: mapping [block]
          "Description" [plain @55:5..55:16]: "The Latest Windows 2016 AMI taken from the public Systems Manager Parameter Store" [plain @55:18..55:99]
          "Type" [plain @56:5..56:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @56:11..56:58]
          "Default" [plain @57:5..57:12]: "/aws/service/ami-windows-latest/Windows_Server-2016-English-Full-Base" [plain @57:14..57:83]
        "LinuxAMIID" [plain @59:3..59:13]: mapping [block]
          "Description" [plain @60:5..60:16]: "The Latest Amazon Linux 2 AMI taken from the public Systems Manager Parameter Store" [plain @60:18..60:101]
          "Type" [plain @61:5..61:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @61:11..61:58]
          "Default" [plain @62:5..62:12]: "/aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2" [plain @62:14..62:75]
        "SubnetId" [plain @64:3..64:11]: mapping [block]
          "Type" [plain @65:5..65:9]: "AWS::EC2::Subnet::Id" [plain @65:11..65:31]
      "Resources" [plain @67:1..67:10]: mapping [block]
        "WindowsInstance" [plain @68:3..68:18]: mapping [block]
          "Type" [plain @69:5..69:9]: "AWS::EC2::Instance" [plain @69:11..69:29]
          "Properties" [plain @70:5..70:15]: mapping [block]
            "ImageId" [plain @71:7..71:14]: "WindowsAMIID" [plain @71:21..71:33] !<!Ref>
            "SubnetId" [plain @72:7..72:15]: "SubnetId" [plain @72:22..72:30] !<!Ref>
            "InstanceType" [plain @73:7..73:19]: "InstanceType" [plain @73:26..73:38] !<!Ref>
            "AvailabilityZone" [plain @74:7..74:23]: "InstanceAZ" [plain @74:30..74:40] !<!Ref>
            "IamInstanceProfile" [plain @75:7..75:25]: "InstanceProfile" [plain @75:32..75:47] !<!Ref>
            "KeyName" [plain @76:7..76:14]: "KeyName" [plain @76:21..76:28] !<!Ref>
            "UserData" [plain @77:7..77:15]: literal [@77:25..88:1] !<!Base64>
              "<powershell>\n"
              "  $AWS_AVAIL_ZONE=(curl http://169.254.169.254/latest/meta-data/placement/availability-zone).Content\n"
              "  $AWS_REGION=$AWS_AVAIL_ZONE.Substring(0,$AWS_AVAIL_ZONE.length-1)\n"
              "  $AWS_INSTANCE_ID=(curl http://169.254.169.254/latest/meta-data/instance-id).Content\n"
              "  $ROOT_VOLUME_IDS=((Get-EC2Instance -Region $AWS_REGION -InstanceId $AWS_INSTANCE_ID).Instances.BlockDeviceMappings | where-object DeviceName -match '/dev/sda1').Ebs.VolumeId\n"
              "  $tag = New-Object Amazon.EC2.Model.Tag\n"
              "  $tag.key = \"MyRootTag\"\n"
              "  $tag.value = \"MyRootVolumesValue\"\n"
              "  New-EC2Tag -Resource $ROOT_VOLUME_IDS -Region $AWS_REGION -Tag $tag\n"
              "</powershell>\n"
            "Tags" [plain @88:7..88:11]: sequence [block]
              - mapping [block]
                "Key" [plain @89:11..89:14]: "Name" [plain @89:16..89:20]
                "Value" [plain @90:11..90:16]: "AWS::StackName" [plain @90:23..90:37] !<!Ref>
            "BlockDeviceMappings" [plain @91:7..91:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @92:11..92:21]: "/dev/sdm" [plain @92:23..92:31]
                "Ebs" [plain @93:11..93:14]: mapping [block]
                  "VolumeType" [plain @94:13..94:23]: "io1" [plain @94:25..94:28]
                  "Iops" [plain @95:13..95:17]: "200" [double-quoted @95:19..95:24]
                  "DeleteOnTermination" [plain @96:13..96:32]: "true" [double-quoted @96:34..96:40]
                  "VolumeSize" [plain @97:13..97:23]: "10" [double-quoted @97:25..97:29]
        "LinuxInstance" [plain @99:3..99:16]: mapping [block]
          "Type" [plain @100:5..100:9]: "AWS::EC2::Instance" [plain @100:11..100:29]
          "Properties" [plain @101:5..101:15]: mapping [block]
            "ImageId" [plain @102:7..102:14]: "LinuxAMIID" [plain @102:21..102:31] !<!Ref>
            "SubnetId" [plain @103:7..103:15]: "SubnetId" [plain @103:22..103:30] !<!Ref>
            "InstanceType" [plain @104:7..104:19]: "InstanceType" [plain @104:26..104:38] !<!Ref>
            "AvailabilityZone" [plain @105:7..105:23]: "InstanceAZ" [plain @105:30..105:40] !<!Ref>
            "IamInstanceProfile" [plain @106:7..106:25]: "InstanceProfile" [plain @106:32..106:47] !<!Ref>
            "KeyName" [plain @107:7..107:14]: "KeyName" [plain @107:21..107:28] !<!Ref>
            "UserData" [plain @108:7..108:15]: literal [@108:25..114:1] !<!Base64>
              "AWS_AVAIL_ZONE=$(curl http://169.254.169.254/latest/meta-data/placement/availability-zone)\n"
              "AWS_REGION=\"`echo \\\"$AWS_AVAIL_ZONE\\\" | sed 's/[a-z]$//'`\"\n"
              "AWS_INSTANCE_ID=$(curl http://169.254.169.254/latest/meta-data/instance-id)\n"
              "ROOT_VOLUME_IDS=$(aws ec2 describe-instances --region $AWS_REGION --instance-id $AWS_INSTANCE_ID --output text --query Reservations[0].Instances[0].BlockDeviceMappings[0].Ebs.VolumeId)\n"
              "aws ec2 create-tags --resources $ROOT_VOLUME_IDS --region $AWS_REGION --tags Key=MyRootTag,Value=MyRootVolumesValue\n"
            "Tags" [plain @114:7..114:11]: sequence [block]
              - mapping [block]
                "Key" [plain @115:11..115:14]: "Name" [plain @115:16..115:20]
                "Value" [plain @116:11..116:16]: "AWS::StackName" [plain @116:23..116:37] !<!Ref>
            "BlockDeviceMappings" [plain @117:7..117:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @118:11..118:21]: "/dev/sdm" [plain @118:23..118:31]
                "Ebs" [plain @119:11..119:14]: mapping [block]
                  "VolumeType" [plain @120:13..120:23]: "io1" [plain @120:25..120:28]
                  "Iops" [plain @121:13..121:17]: "200" [double-quoted @121:19..121:24]
                  "DeleteOnTermination" [plain @122:13..122:32]: "true" [double-quoted @122:34..122:40]
                  "VolumeSize" [plain @123:13..123:23]: "10" [double-quoted @123:25..123:29]
        "InstanceRole" [plain @125:3..125:15]: mapping [block]
          "Type" [plain @126:5..126:9]: "AWS::IAM::Role" [plain @126:11..126:25]
          "Properties" [plain @127:5..127:15]: mapping [block]
            "AssumeRolePolicyDocument" [plain @128:7..128:31]: mapping [block]
              "Version" [plain @129:9..129:16]: "2012-10-17" [double-quoted @129:18..129:30]
              "Statement" [plain @130:9..130:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @131:13..131:19]: "Allow" [plain @131:21..131:26]
                  "Principal" [plain @132:13..132:22]: mapping [block]
                    "Service" [plain @133:15..133:22]: sequence [block]
                      - "ec2.amazonaws.com" [plain @134:19..134:36]
                  "Action" [plain @135:13..135:19]: sequence [block]
                    - "sts:AssumeRole" [plain @136:17..136:31]
            "Path" [plain @137:7..137:11]: "/" [plain @137:13..137:14]
            "Policies" [plain @138:7..138:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @139:11..139:21]: "taginstancepolicy" [plain @139:23..139:40]
                "PolicyDocument" [plain @140:11..140:25]: mapping [block]
                  "Version" [plain @141:13..141:20]: "2012-10-17" [double-quoted @141:22..141:34]
                  "Statement" [plain @142:13..142:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @143:17..143:23]: "Allow" [plain @143:25..143:30]
                      "Action" [plain @144:17..144:23]: sequence [block]
                        - "ec2:Describe*" [plain @145:21..145:34]
                        - "ec2:CreateTags" [plain @146:21..146:35]
                      "Resource" [plain @147:17..147:25]: "*" [single-quoted @147:27..147:30]
        "InstanceProfile" [plain @149:3..149:18]: mapping [block]
          "Type" [plain @150:5..150:9]: "AWS::IAM::InstanceProfile" [plain @150:11..150:36]
          "Properties" [plain @151:5..151:15]: mapping [block]
            "Path" [plain @152:7..152:11]: "/" [plain @152:13..152:14]
            "Roles" [plain @153:7..153:12]: sequence [block]
              - "InstanceRole" [plain @154:16..154:28] !<!Ref>
    "#);
}
