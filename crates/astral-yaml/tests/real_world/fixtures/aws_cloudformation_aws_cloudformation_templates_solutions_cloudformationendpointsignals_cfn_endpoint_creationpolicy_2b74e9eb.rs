#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-cloudformationendpointsignals-cfn-endpoint-creationpolicy-2b74e9eb"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: literal [@3:14..6:1]
        "This template deploys a VPC with a pair of public and private subnets spread across two Availabilty Zones. It deploys an Internet Gateway, with a default route on the public subnets and a bastion server. It deploys a VPC Endpoint for CloudFormation so an instance in the private subnet can use cfn-signal for its CreationPolicy. **WARNING** You will be billed for the AWS resources used if you create a stack from this template.\n"
      "Parameters" [plain @6:1..6:11]: mapping [block]
        "EnvironmentName" [plain @7:3..7:18]: mapping [block]
          "Description" [plain @8:5..8:16]: "An environment name that will be prefixed to resource names" [plain @8:18..8:77]
          "Type" [plain @9:5..9:9]: "String" [plain @9:11..9:17]
        "VpcCIDR" [plain @11:3..11:10]: mapping [block]
          "Description" [plain @12:5..12:16]: "Please enter the IP range (CIDR notation) for this VPC" [plain @12:18..12:72]
          "Type" [plain @13:5..13:9]: "String" [plain @13:11..13:17]
          "Default" [plain @14:5..14:12]: "10.192.0.0/16" [plain @14:14..14:27]
          "AllowedPattern" [plain @15:5..15:19]: "((\\d{1,3})\\.){3}\\d{1,3}/\\d{1,2}" [plain @15:21..15:52]
        "PublicSubnet1CIDR" [plain @17:3..17:20]: mapping [block]
          "Description" [plain @18:5..18:16]: "Please enter the IP range (CIDR notation) for the public subnet in the first Availability Zone" [plain @18:18..18:112]
          "Type" [plain @19:5..19:9]: "String" [plain @19:11..19:17]
          "Default" [plain @20:5..20:12]: "10.192.10.0/24" [plain @20:14..20:28]
          "AllowedPattern" [plain @21:5..21:19]: "((\\d{1,3})\\.){3}\\d{1,3}/\\d{1,2}" [plain @21:21..21:52]
        "PublicSubnet2CIDR" [plain @23:3..23:20]: mapping [block]
          "Description" [plain @24:5..24:16]: "Please enter the IP range (CIDR notation) for the public subnet in the second Availability Zone" [plain @24:18..24:113]
          "Type" [plain @25:5..25:9]: "String" [plain @25:11..25:17]
          "Default" [plain @26:5..26:12]: "10.192.11.0/24" [plain @26:14..26:28]
          "AllowedPattern" [plain @27:5..27:19]: "((\\d{1,3})\\.){3}\\d{1,3}/\\d{1,2}" [plain @27:21..27:52]
        "PrivateSubnet1CIDR" [plain @29:3..29:21]: mapping [block]
          "Description" [plain @30:5..30:16]: "Please enter the IP range (CIDR notation) for the private subnet in the first Availability Zone" [plain @30:18..30:113]
          "Type" [plain @31:5..31:9]: "String" [plain @31:11..31:17]
          "Default" [plain @32:5..32:12]: "10.192.20.0/24" [plain @32:14..32:28]
          "AllowedPattern" [plain @33:5..33:19]: "((\\d{1,3})\\.){3}\\d{1,3}/\\d{1,2}" [plain @33:21..33:52]
        "PrivateSubnet2CIDR" [plain @35:3..35:21]: mapping [block]
          "Description" [plain @36:5..36:16]: "Please enter the IP range (CIDR notation) for the private subnet in the second Availability Zone" [plain @36:18..36:114]
          "Type" [plain @37:5..37:9]: "String" [plain @37:11..37:17]
          "Default" [plain @38:5..38:12]: "10.192.21.0/24" [plain @38:14..38:28]
          "AllowedPattern" [plain @39:5..39:19]: "((\\d{1,3})\\.){3}\\d{1,3}/\\d{1,2}" [plain @39:21..39:52]
        "KeyName" [plain @41:3..41:10]: mapping [block]
          "Description" [plain @42:5..42:16]: "Key pair for EC2 access" [plain @42:18..42:41]
          "Type" [plain @43:5..43:9]: "AWS::EC2::KeyPair::KeyName" [plain @43:11..43:37]
        "LinuxAMI" [plain @45:3..45:11]: mapping [block]
          "Description" [plain @46:5..46:16]: "Managed AMI ID for Amazon Linux" [plain @46:18..46:49]
          "Type" [plain @47:5..47:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @47:11..47:58]
          "Default" [plain @48:5..48:12]: "/aws/service/ami-amazon-linux-latest/amzn-ami-hvm-x86_64-gp2" [plain @48:14..48:74]
      "Resources" [plain @50:1..50:10]: mapping [block]
        "VPC" [plain @51:3..51:6]: mapping [block]
          "Type" [plain @52:5..52:9]: "AWS::EC2::VPC" [plain @52:11..52:24]
          "Properties" [plain @53:5..53:15]: mapping [block]
            "EnableDnsSupport" [plain @54:7..54:23]: "true" [plain @54:25..54:29]
            "EnableDnsHostnames" [plain @55:7..55:25]: "true" [plain @55:27..55:31]
            "CidrBlock" [plain @56:7..56:16]: "VpcCIDR" [plain @56:23..56:30] !<!Ref>
            "Tags" [plain @57:7..57:11]: sequence [block]
              - mapping [block]
                "Key" [plain @58:11..58:14]: "Name" [plain @58:16..58:20]
                "Value" [plain @59:11..59:16]: "EnvironmentName" [plain @59:23..59:38] !<!Ref>
        "InternetGateway" [plain @61:3..61:18]: mapping [block]
          "Type" [plain @62:5..62:9]: "AWS::EC2::InternetGateway" [plain @62:11..62:36]
          "Properties" [plain @63:5..63:15]: mapping [block]
            "Tags" [plain @64:7..64:11]: sequence [block]
              - mapping [block]
                "Key" [plain @65:11..65:14]: "Name" [plain @65:16..65:20]
                "Value" [plain @66:11..66:16]: "EnvironmentName" [plain @66:23..66:38] !<!Ref>
        "InternetGatewayAttachment" [plain @68:3..68:28]: mapping [block]
          "Type" [plain @69:5..69:9]: "AWS::EC2::VPCGatewayAttachment" [plain @69:11..69:41]
          "Properties" [plain @70:5..70:15]: mapping [block]
            "InternetGatewayId" [plain @71:7..71:24]: "InternetGateway" [plain @71:31..71:46] !<!Ref>
            "VpcId" [plain @72:7..72:12]: "VPC" [plain @72:19..72:22] !<!Ref>
        "CfnEndpoint" [plain @76:3..76:14]: mapping [block]
          "Type" [plain @77:5..77:9]: "AWS::EC2::VPCEndpoint" [plain @77:11..77:32]
          "Properties" [plain @78:5..78:15]: mapping [block]
            "VpcId" [plain @79:7..79:12]: "VPC" [plain @79:19..79:22] !<!Ref>
            "ServiceName" [plain @80:7..80:18]: "com.amazonaws.${AWS::Region}.cloudformation" [plain @80:25..80:68] !<!Sub>
            "VpcEndpointType" [plain @81:7..81:22]: "Interface" [plain @81:24..81:33]
            "PrivateDnsEnabled" [plain @82:7..82:24]: "true" [plain @82:26..82:30]
            "SubnetIds" [plain @83:7..83:16]: sequence [block]
              - "PrivateSubnet1" [plain @84:16..84:30] !<!Ref>
              - "PrivateSubnet2" [plain @85:16..85:30] !<!Ref>
            "SecurityGroupIds" [plain @86:7..86:23]: sequence [block]
              - "EndpointSG" [plain @87:16..87:26] !<!Ref>
        "PublicSubnet1" [plain @89:3..89:16]: mapping [block]
          "Type" [plain @90:5..90:9]: "AWS::EC2::Subnet" [plain @90:11..90:27]
          "Properties" [plain @91:5..91:15]: mapping [block]
            "VpcId" [plain @92:7..92:12]: "VPC" [plain @92:19..92:22] !<!Ref>
            "AvailabilityZone" [plain @93:7..93:23]: sequence [block] !<!Select>
              - "0" [plain @94:11..94:12]
              - "" [plain @95:18..95:18] !<!GetAZs>
            "CidrBlock" [plain @96:7..96:16]: "PublicSubnet1CIDR" [plain @96:23..96:40] !<!Ref>
            "MapPublicIpOnLaunch" [plain @97:7..97:26]: "true" [plain @97:28..97:32]
            "Tags" [plain @98:7..98:11]: sequence [block]
              - mapping [block]
                "Key" [plain @99:11..99:14]: "Name" [plain @99:16..99:20]
                "Value" [plain @100:11..100:16]: "${EnvironmentName} Public Subnet (AZ1)" [plain @100:23..100:61] !<!Sub>
        "PublicSubnet2" [plain @102:3..102:16]: mapping [block]
          "Type" [plain @103:5..103:9]: "AWS::EC2::Subnet" [plain @103:11..103:27]
          "Properties" [plain @104:5..104:15]: mapping [block]
            "VpcId" [plain @105:7..105:12]: "VPC" [plain @105:19..105:22] !<!Ref>
            "AvailabilityZone" [plain @106:7..106:23]: sequence [block] !<!Select>
              - "1" [plain @107:11..107:12]
              - "" [plain @108:18..108:18] !<!GetAZs>
            "CidrBlock" [plain @109:7..109:16]: "PublicSubnet2CIDR" [plain @109:23..109:40] !<!Ref>
            "MapPublicIpOnLaunch" [plain @110:7..110:26]: "true" [plain @110:28..110:32]
            "Tags" [plain @111:7..111:11]: sequence [block]
              - mapping [block]
                "Key" [plain @112:11..112:14]: "Name" [plain @112:16..112:20]
                "Value" [plain @113:11..113:16]: "${EnvironmentName} Public Subnet (AZ2)" [plain @113:23..113:61] !<!Sub>
        "PrivateSubnet1" [plain @115:3..115:17]: mapping [block]
          "Type" [plain @116:5..116:9]: "AWS::EC2::Subnet" [plain @116:11..116:27]
          "Properties" [plain @117:5..117:15]: mapping [block]
            "VpcId" [plain @118:7..118:12]: "VPC" [plain @118:19..118:22] !<!Ref>
            "AvailabilityZone" [plain @119:7..119:23]: sequence [block] !<!Select>
              - "0" [plain @120:11..120:12]
              - "" [plain @121:18..121:18] !<!GetAZs>
            "CidrBlock" [plain @122:7..122:16]: "PrivateSubnet1CIDR" [plain @122:23..122:41] !<!Ref>
            "MapPublicIpOnLaunch" [plain @123:7..123:26]: "false" [plain @123:28..123:33]
            "Tags" [plain @124:7..124:11]: sequence [block]
              - mapping [block]
                "Key" [plain @125:11..125:14]: "Name" [plain @125:16..125:20]
                "Value" [plain @126:11..126:16]: "${EnvironmentName} Private Subnet (AZ1)" [plain @126:23..126:62] !<!Sub>
        "PrivateSubnet2" [plain @128:3..128:17]: mapping [block]
          "Type" [plain @129:5..129:9]: "AWS::EC2::Subnet" [plain @129:11..129:27]
          "Properties" [plain @130:5..130:15]: mapping [block]
            "VpcId" [plain @131:7..131:12]: "VPC" [plain @131:19..131:22] !<!Ref>
            "AvailabilityZone" [plain @132:7..132:23]: sequence [block] !<!Select>
              - "1" [plain @133:11..133:12]
              - "" [plain @134:18..134:18] !<!GetAZs>
            "CidrBlock" [plain @135:7..135:16]: "PrivateSubnet2CIDR" [plain @135:23..135:41] !<!Ref>
            "MapPublicIpOnLaunch" [plain @136:7..136:26]: "false" [plain @136:28..136:33]
            "Tags" [plain @137:7..137:11]: sequence [block]
              - mapping [block]
                "Key" [plain @138:11..138:14]: "Name" [plain @138:16..138:20]
                "Value" [plain @139:11..139:16]: "${EnvironmentName} Private Subnet (AZ2)" [plain @139:23..139:62] !<!Sub>
        "PublicRouteTable" [plain @141:3..141:19]: mapping [block]
          "Type" [plain @142:5..142:9]: "AWS::EC2::RouteTable" [plain @142:11..142:31]
          "Properties" [plain @143:5..143:15]: mapping [block]
            "VpcId" [plain @144:7..144:12]: "VPC" [plain @144:19..144:22] !<!Ref>
            "Tags" [plain @145:7..145:11]: sequence [block]
              - mapping [block]
                "Key" [plain @146:11..146:14]: "Name" [plain @146:16..146:20]
                "Value" [plain @147:11..147:16]: "${EnvironmentName} Public Routes" [plain @147:23..147:55] !<!Sub>
        "DefaultPublicRoute" [plain @149:3..149:21]: mapping [block]
          "Type" [plain @150:5..150:9]: "AWS::EC2::Route" [plain @150:11..150:26]
          "DependsOn" [plain @151:5..151:14]: "InternetGatewayAttachment" [plain @151:16..151:41]
          "Properties" [plain @152:5..152:15]: mapping [block]
            "RouteTableId" [plain @153:7..153:19]: "PublicRouteTable" [plain @153:26..153:42] !<!Ref>
            "DestinationCidrBlock" [plain @154:7..154:27]: "0.0.0.0/0" [plain @154:29..154:38]
            "GatewayId" [plain @155:7..155:16]: "InternetGateway" [plain @155:23..155:38] !<!Ref>
        "PublicSubnet1RouteTableAssociation" [plain @157:3..157:37]: mapping [block]
          "Type" [plain @158:5..158:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @158:11..158:48]
          "Properties" [plain @159:5..159:15]: mapping [block]
            "RouteTableId" [plain @160:7..160:19]: "PublicRouteTable" [plain @160:26..160:42] !<!Ref>
            "SubnetId" [plain @161:7..161:15]: "PublicSubnet1" [plain @161:22..161:35] !<!Ref>
        "PublicSubnet2RouteTableAssociation" [plain @163:3..163:37]: mapping [block]
          "Type" [plain @164:5..164:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @164:11..164:48]
          "Properties" [plain @165:5..165:15]: mapping [block]
            "RouteTableId" [plain @166:7..166:19]: "PublicRouteTable" [plain @166:26..166:42] !<!Ref>
            "SubnetId" [plain @167:7..167:15]: "PublicSubnet2" [plain @167:22..167:35] !<!Ref>
        "PrivateRouteTable1" [plain @169:3..169:21]: mapping [block]
          "Type" [plain @170:5..170:9]: "AWS::EC2::RouteTable" [plain @170:11..170:31]
          "Properties" [plain @171:5..171:15]: mapping [block]
            "VpcId" [plain @172:7..172:12]: "VPC" [plain @172:19..172:22] !<!Ref>
            "Tags" [plain @173:7..173:11]: sequence [block]
              - mapping [block]
                "Key" [plain @174:11..174:14]: "Name" [plain @174:16..174:20]
                "Value" [plain @175:11..175:16]: "${EnvironmentName} Private Routes (AZ1)" [plain @175:23..175:62] !<!Sub>
        "PrivateSubnet1RouteTableAssociation" [plain @177:3..177:38]: mapping [block]
          "Type" [plain @178:5..178:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @178:11..178:48]
          "Properties" [plain @179:5..179:15]: mapping [block]
            "RouteTableId" [plain @180:7..180:19]: "PrivateRouteTable1" [plain @180:26..180:44] !<!Ref>
            "SubnetId" [plain @181:7..181:15]: "PrivateSubnet1" [plain @181:22..181:36] !<!Ref>
        "PrivateRouteTable2" [plain @183:3..183:21]: mapping [block]
          "Type" [plain @184:5..184:9]: "AWS::EC2::RouteTable" [plain @184:11..184:31]
          "Properties" [plain @185:5..185:15]: mapping [block]
            "VpcId" [plain @186:7..186:12]: "VPC" [plain @186:19..186:22] !<!Ref>
            "Tags" [plain @187:7..187:11]: sequence [block]
              - mapping [block]
                "Key" [plain @188:11..188:14]: "Name" [plain @188:16..188:20]
                "Value" [plain @189:11..189:16]: "${EnvironmentName} Private Routes (AZ2)" [plain @189:23..189:62] !<!Sub>
        "PrivateSubnet2RouteTableAssociation" [plain @191:3..191:38]: mapping [block]
          "Type" [plain @192:5..192:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @192:11..192:48]
          "Properties" [plain @193:5..193:15]: mapping [block]
            "RouteTableId" [plain @194:7..194:19]: "PrivateRouteTable2" [plain @194:26..194:44] !<!Ref>
            "SubnetId" [plain @195:7..195:15]: "PrivateSubnet2" [plain @195:22..195:36] !<!Ref>
        "RootRole" [plain @197:3..197:11]: mapping [block]
          "Type" [plain @198:5..198:9]: "AWS::IAM::Role" [plain @198:11..198:25]
          "Properties" [plain @199:5..199:15]: mapping [block]
            "AssumeRolePolicyDocument" [plain @200:7..200:31]: mapping [block]
              "Version" [plain @201:9..201:16]: "2012-10-17" [double-quoted @201:18..201:30]
              "Statement" [plain @202:9..202:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @203:13..203:19]: "Allow" [plain @203:21..203:26]
                  "Principal" [plain @204:13..204:22]: mapping [block]
                    "Service" [plain @205:15..205:22]: sequence [block]
                      - "ec2.amazonaws.com" [plain @206:19..206:36]
                  "Action" [plain @207:13..207:19]: sequence [block]
                    - "sts:AssumeRole" [plain @208:17..208:31]
            "Path" [plain @209:7..209:11]: "/" [plain @209:13..209:14]
            "Policies" [plain @210:7..210:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @211:11..211:21]: "root" [plain @211:23..211:27]
                "PolicyDocument" [plain @212:11..212:25]: mapping [block]
                  "Version" [plain @213:13..213:20]: "2012-10-17" [double-quoted @213:22..213:34]
                  "Statement" [plain @214:13..214:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @215:17..215:23]: "Allow" [plain @215:25..215:30]
                      "Action" [plain @216:17..216:23]: "cloudformation:*" [plain @216:25..216:41]
                      "Resource" [plain @217:17..217:25]: "*" [single-quoted @217:27..217:30]
        "BastionInstance" [plain @219:3..219:18]: mapping [block]
          "Type" [plain @220:5..220:9]: "AWS::EC2::Instance" [plain @220:11..220:29]
          "Properties" [plain @221:5..221:15]: mapping [block]
            "KeyName" [plain @222:7..222:14]: "KeyName" [plain @222:21..222:28] !<!Ref>
            "InstanceType" [plain @223:7..223:19]: "t2.micro" [plain @223:21..223:29]
            "SecurityGroupIds" [plain @224:7..224:23]: sequence [block]
              - "BastionSG" [plain @225:16..225:25] !<!Ref>
            "SubnetId" [plain @226:7..226:15]: "PublicSubnet1" [plain @226:22..226:35] !<!Ref>
            "ImageId" [plain @227:7..227:14]: "LinuxAMI" [plain @227:21..227:29] !<!Ref>
            "IamInstanceProfile" [plain @228:7..228:25]: "BastionProfile" [plain @228:32..228:46] !<!Ref>
            "Tags" [plain @229:7..229:11]: sequence [block]
              - mapping [block]
                "Key" [plain @230:11..230:14]: "Name" [plain @230:16..230:20]
                "Value" [plain @231:11..231:16]: "Bastion" [plain @231:18..231:25]
        "BastionSG" [plain @233:3..233:12]: mapping [block]
          "Type" [plain @234:5..234:9]: "AWS::EC2::SecurityGroup" [plain @234:11..234:34]
          "Metadata" [plain @235:5..235:13]: mapping [block]
            "guard" [plain @236:7..236:12]: mapping [block]
              "SuppressedRules" [plain @237:9..237:24]: sequence [block]
                - "INCOMING_SSH_DISABLED" [plain @238:13..238:34]
          "Properties" [plain @239:5..239:15]: mapping [block]
            "GroupDescription" [plain @240:7..240:23]: "Inbound Bastion Traffic" [plain @240:25..240:48]
            "SecurityGroupIngress" [plain @241:7..241:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @242:11..242:21]: "tcp" [plain @242:23..242:26]
                "FromPort" [plain @243:11..243:19]: "22" [plain @243:21..243:23]
                "ToPort" [plain @244:11..244:17]: "22" [plain @244:19..244:21]
                "CidrIp" [plain @245:11..245:17]: "0.0.0.0/0" [plain @245:19..245:28]
            "VpcId" [plain @246:7..246:12]: "VPC" [plain @246:19..246:22] !<!Ref>
            "Tags" [plain @247:7..247:11]: sequence [block]
              - mapping [block]
                "Key" [plain @248:11..248:14]: "Name" [plain @248:16..248:20]
                "Value" [plain @249:11..249:16]: "BastionSG" [plain @249:18..249:27]
        "BastionProfile" [plain @251:3..251:17]: mapping [block]
          "Type" [plain @252:5..252:9]: "AWS::IAM::InstanceProfile" [plain @252:11..252:36]
          "Properties" [plain @253:5..253:15]: mapping [block]
            "Path" [plain @254:7..254:11]: "/" [plain @254:13..254:14]
            "Roles" [plain @255:7..255:12]: sequence [block]
              - "RootRole" [plain @256:16..256:24] !<!Ref>
        "PrivateInstance" [plain @258:3..258:18]: mapping [block]
          "CreationPolicy" [plain @259:5..259:19]: mapping [block]
            "ResourceSignal" [plain @260:7..260:21]: mapping [block]
              "Count" [plain @261:9..261:14]: "1" [plain @261:16..261:17]
              "Timeout" [plain @262:9..262:16]: "PT15M" [plain @262:18..262:23]
          "Type" [plain @263:5..263:9]: "AWS::EC2::Instance" [plain @263:11..263:29]
          "DependsOn" [plain @264:5..264:14]: "CfnEndpoint" [plain @264:16..264:27]
          "Properties" [plain @265:5..265:15]: mapping [block]
            "KeyName" [plain @266:7..266:14]: "KeyName" [plain @266:21..266:28] !<!Ref>
            "InstanceType" [plain @267:7..267:19]: "t2.micro" [plain @267:21..267:29]
            "SecurityGroupIds" [plain @268:7..268:23]: sequence [block]
              - "PrivateSG" [plain @269:16..269:25] !<!Ref>
            "SubnetId" [plain @270:7..270:15]: "PrivateSubnet1" [plain @270:22..270:36] !<!Ref>
            "ImageId" [plain @271:7..271:14]: "LinuxAMI" [plain @271:21..271:29] !<!Ref>
            "IamInstanceProfile" [plain @272:7..272:25]: "PrivateProfile" [plain @272:32..272:46] !<!Ref>
            "UserData" [plain @273:7..273:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @274:9..274:16]: literal [@274:18..280:1]
                "#!/bin/bash -x\n"
                "date > /tmp/datefile\n"
                "cat /tmp/datefile\n"
                "# Signal the status from instance\n"
                "/opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} --resource PrivateInstance --region ${AWS::Region}\n"
            "Tags" [plain @280:7..280:11]: sequence [block]
              - mapping [block]
                "Key" [plain @281:11..281:14]: "Name" [plain @281:16..281:20]
                "Value" [plain @282:11..282:16]: "Private" [plain @282:18..282:25]
        "PrivateSG" [plain @284:3..284:12]: mapping [block]
          "Type" [plain @285:5..285:9]: "AWS::EC2::SecurityGroup" [plain @285:11..285:34]
          "Properties" [plain @286:5..286:15]: mapping [block]
            "GroupDescription" [plain @287:7..287:23]: "Traffic from Bastion" [plain @287:25..287:45]
            "SecurityGroupIngress" [plain @288:7..288:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @289:11..289:21]: "tcp" [plain @289:23..289:26]
                "FromPort" [plain @290:11..290:19]: "22" [plain @290:21..290:23]
                "ToPort" [plain @291:11..291:17]: "22" [plain @291:19..291:21]
                "SourceSecurityGroupId" [plain @292:11..292:32]: "BastionSG" [plain @292:39..292:48] !<!Ref>
            "VpcId" [plain @293:7..293:12]: "VPC" [plain @293:19..293:22] !<!Ref>
            "Tags" [plain @294:7..294:11]: sequence [block]
              - mapping [block]
                "Key" [plain @295:11..295:14]: "Name" [plain @295:16..295:20]
                "Value" [plain @296:11..296:16]: "PrivateSG" [plain @296:18..296:27]
        "PrivateProfile" [plain @298:3..298:17]: mapping [block]
          "Type" [plain @299:5..299:9]: "AWS::IAM::InstanceProfile" [plain @299:11..299:36]
          "Properties" [plain @300:5..300:15]: mapping [block]
            "Path" [plain @301:7..301:11]: "/" [plain @301:13..301:14]
            "Roles" [plain @302:7..302:12]: sequence [block]
              - "RootRole" [plain @303:16..303:24] !<!Ref>
        "EndpointSG" [plain @305:3..305:13]: mapping [block]
          "Type" [plain @306:5..306:9]: "AWS::EC2::SecurityGroup" [plain @306:11..306:34]
          "Properties" [plain @307:5..307:15]: mapping [block]
            "GroupDescription" [plain @308:7..308:23]: "Traffic into CloudFormation Endpoint" [plain @308:25..308:61]
            "SecurityGroupIngress" [plain @309:7..309:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @310:11..310:21]: "tcp" [plain @310:23..310:26]
                "FromPort" [plain @311:11..311:19]: "443" [plain @311:21..311:24]
                "ToPort" [plain @312:11..312:17]: "443" [plain @312:19..312:22]
                "CidrIp" [plain @313:11..313:17]: "0.0.0.0/0" [plain @313:19..313:28]
            "VpcId" [plain @314:7..314:12]: "VPC" [plain @314:19..314:22] !<!Ref>
            "Tags" [plain @315:7..315:11]: sequence [block]
              - mapping [block]
                "Key" [plain @316:11..316:14]: "Name" [plain @316:16..316:20]
                "Value" [plain @317:11..317:16]: "EndpointSG" [plain @317:18..317:28]
      "Outputs" [plain @319:1..319:8]: mapping [block]
        "VPC" [plain @320:3..320:6]: mapping [block]
          "Description" [plain @321:5..321:16]: "A reference to the created VPC" [plain @321:18..321:48]
          "Value" [plain @322:5..322:10]: "VPC" [plain @322:17..322:20] !<!Ref>
        "PublicSubnets" [plain @324:3..324:16]: mapping [block]
          "Description" [plain @325:5..325:16]: "A list of the public subnets" [plain @325:18..325:46]
          "Value" [plain @326:5..326:10]: sequence [block] !<!Join>
            - "," [single-quoted @327:9..327:12]
            - sequence [block]
              - "PublicSubnet1" [plain @328:16..328:29] !<!Ref>
              - "PublicSubnet2" [plain @329:16..329:29] !<!Ref>
        "PrivateSubnets" [plain @331:3..331:17]: mapping [block]
          "Description" [plain @332:5..332:16]: "A list of the private subnets" [plain @332:18..332:47]
          "Value" [plain @333:5..333:10]: sequence [block] !<!Join>
            - "," [single-quoted @334:9..334:12]
            - sequence [block]
              - "PrivateSubnet1" [plain @335:16..335:30] !<!Ref>
              - "PrivateSubnet2" [plain @336:16..336:30] !<!Ref>
        "CfnEndpoint" [plain @338:3..338:14]: mapping [block]
          "Description" [plain @339:5..339:16]: "A reference to the CloudFormation Endpoint used for signaling from the private instance" [plain @339:18..339:105]
          "Value" [plain @340:5..340:10]: "CfnEndpoint" [plain @340:17..340:28] !<!Ref>
    "##);
}
