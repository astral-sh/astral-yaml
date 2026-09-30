#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-ecs-fargatelaunchtype-clusters-private-vpc-4501c05f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "This stack deploys a Fargate cluster that is in a VPC with both public and private subnets. Containers can be deployed into either the public subnets or the private subnets, and there are two load balancers. One is inside the public subnet, which can be used to send traffic to the containers in the private subnet, and one in the private subnet, which can be used for private internal traffic between internal services." [plain @3:14..3:434]
      "Mappings" [plain @5:1..5:9]: mapping [block]
        "SubnetConfig" [plain @20:3..20:15]: mapping [block]
          "VPC" [plain @21:5..21:8]: mapping [block]
            "CIDR" [plain @22:7..22:11]: "10.0.0.0/16" [plain @22:13..22:24]
          "PublicOne" [plain @23:5..23:14]: mapping [block]
            "CIDR" [plain @24:7..24:11]: "10.0.0.0/24" [plain @24:13..24:24]
          "PublicTwo" [plain @25:5..25:14]: mapping [block]
            "CIDR" [plain @26:7..26:11]: "10.0.1.0/24" [plain @26:13..26:24]
          "PrivateOne" [plain @27:5..27:15]: mapping [block]
            "CIDR" [plain @28:7..28:11]: "10.0.2.0/24" [plain @28:13..28:24]
          "PrivateTwo" [plain @29:5..29:15]: mapping [block]
            "CIDR" [plain @30:7..30:11]: "10.0.3.0/24" [plain @30:13..30:24]
      "Resources" [plain @32:1..32:10]: mapping [block]
        "VPC" [plain @38:3..38:6]: mapping [block]
          "Type" [plain @39:5..39:9]: "AWS::EC2::VPC" [plain @39:11..39:24]
          "Properties" [plain @40:5..40:15]: mapping [block]
            "EnableDnsSupport" [plain @41:7..41:23]: "true" [plain @41:25..41:29]
            "EnableDnsHostnames" [plain @42:7..42:25]: "true" [plain @42:27..42:31]
            "CidrBlock" [plain @43:7..43:16]: sequence [block] !<!FindInMap>
              - "SubnetConfig" [plain @44:11..44:23]
              - "VPC" [plain @45:11..45:14]
              - "CIDR" [plain @46:11..46:15]
        "PublicSubnetOne" [plain @49:3..49:18]: mapping [block]
          "Type" [plain @50:5..50:9]: "AWS::EC2::Subnet" [plain @50:11..50:27]
          "Properties" [plain @51:5..51:15]: mapping [block]
            "AvailabilityZone" [plain @52:7..52:23]: sequence [block] !<!Select>
              - "0" [plain @53:11..53:12]
              - mapping [block] !<!GetAZs>
                "Ref" [plain @55:11..55:14]: "AWS::Region" [plain @55:16..55:27]
            "VpcId" [plain @56:7..56:12]: "VPC" [plain @56:19..56:22] !<!Ref>
            "CidrBlock" [plain @57:7..57:16]: sequence [block] !<!FindInMap>
              - "SubnetConfig" [plain @58:11..58:23]
              - "PublicOne" [plain @59:11..59:20]
              - "CIDR" [plain @60:11..60:15]
            "MapPublicIpOnLaunch" [plain @61:7..61:26]: "true" [plain @61:28..61:32]
        "PublicSubnetTwo" [plain @63:3..63:18]: mapping [block]
          "Type" [plain @64:5..64:9]: "AWS::EC2::Subnet" [plain @64:11..64:27]
          "Properties" [plain @65:5..65:15]: mapping [block]
            "AvailabilityZone" [plain @66:7..66:23]: sequence [block] !<!Select>
              - "1" [plain @67:11..67:12]
              - mapping [block] !<!GetAZs>
                "Ref" [plain @69:11..69:14]: "AWS::Region" [plain @69:16..69:27]
            "VpcId" [plain @70:7..70:12]: "VPC" [plain @70:19..70:22] !<!Ref>
            "CidrBlock" [plain @71:7..71:16]: sequence [block] !<!FindInMap>
              - "SubnetConfig" [plain @72:11..72:23]
              - "PublicTwo" [plain @73:11..73:20]
              - "CIDR" [plain @74:11..74:15]
            "MapPublicIpOnLaunch" [plain @75:7..75:26]: "true" [plain @75:28..75:32]
        "PrivateSubnetOne" [plain @80:3..80:19]: mapping [block]
          "Type" [plain @81:5..81:9]: "AWS::EC2::Subnet" [plain @81:11..81:27]
          "Properties" [plain @82:5..82:15]: mapping [block]
            "AvailabilityZone" [plain @83:7..83:23]: sequence [block] !<!Select>
              - "0" [plain @84:11..84:12]
              - mapping [block] !<!GetAZs>
                "Ref" [plain @86:11..86:14]: "AWS::Region" [plain @86:16..86:27]
            "VpcId" [plain @87:7..87:12]: "VPC" [plain @87:19..87:22] !<!Ref>
            "CidrBlock" [plain @88:7..88:16]: sequence [block] !<!FindInMap>
              - "SubnetConfig" [plain @89:11..89:23]
              - "PrivateOne" [plain @90:11..90:21]
              - "CIDR" [plain @91:11..91:15]
        "PrivateSubnetTwo" [plain @93:3..93:19]: mapping [block]
          "Type" [plain @94:5..94:9]: "AWS::EC2::Subnet" [plain @94:11..94:27]
          "Properties" [plain @95:5..95:15]: mapping [block]
            "AvailabilityZone" [plain @96:7..96:23]: sequence [block] !<!Select>
              - "1" [plain @97:11..97:12]
              - mapping [block] !<!GetAZs>
                "Ref" [plain @99:11..99:14]: "AWS::Region" [plain @99:16..99:27]
            "VpcId" [plain @100:7..100:12]: "VPC" [plain @100:19..100:22] !<!Ref>
            "CidrBlock" [plain @101:7..101:16]: sequence [block] !<!FindInMap>
              - "SubnetConfig" [plain @102:11..102:23]
              - "PrivateTwo" [plain @103:11..103:21]
              - "CIDR" [plain @104:11..104:15]
        "InternetGateway" [plain @109:3..109:18]: mapping [block]
          "Type" [plain @110:5..110:9]: "AWS::EC2::InternetGateway" [plain @110:11..110:36]
        "GatewayAttachement" [plain @112:3..112:21]: mapping [block]
          "Type" [plain @113:5..113:9]: "AWS::EC2::VPCGatewayAttachment" [plain @113:11..113:41]
          "Properties" [plain @114:5..114:15]: mapping [block]
            "VpcId" [plain @115:7..115:12]: "VPC" [plain @115:19..115:22] !<!Ref>
            "InternetGatewayId" [plain @116:7..116:24]: "InternetGateway" [plain @116:31..116:46] !<!Ref>
        "PublicRouteTable" [plain @118:3..118:19]: mapping [block]
          "Type" [plain @119:5..119:9]: "AWS::EC2::RouteTable" [plain @119:11..119:31]
          "Properties" [plain @120:5..120:15]: mapping [block]
            "VpcId" [plain @121:7..121:12]: "VPC" [plain @121:19..121:22] !<!Ref>
        "PublicRoute" [plain @123:3..123:14]: mapping [block]
          "Type" [plain @124:5..124:9]: "AWS::EC2::Route" [plain @124:11..124:26]
          "DependsOn" [plain @125:5..125:14]: "GatewayAttachement" [plain @125:16..125:34]
          "Properties" [plain @126:5..126:15]: mapping [block]
            "RouteTableId" [plain @127:7..127:19]: "PublicRouteTable" [plain @127:26..127:42] !<!Ref>
            "DestinationCidrBlock" [plain @128:7..128:27]: "0.0.0.0/0" [plain @128:29..128:38]
            "GatewayId" [plain @129:7..129:16]: "InternetGateway" [plain @129:23..129:38] !<!Ref>
        "PublicSubnetOneRouteTableAssociation" [plain @131:3..131:39]: mapping [block]
          "Type" [plain @132:5..132:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @132:11..132:48]
          "Properties" [plain @133:5..133:15]: mapping [block]
            "SubnetId" [plain @134:7..134:15]: "PublicSubnetOne" [plain @134:22..134:37] !<!Ref>
            "RouteTableId" [plain @135:7..135:19]: "PublicRouteTable" [plain @135:26..135:42] !<!Ref>
        "PublicSubnetTwoRouteTableAssociation" [plain @137:3..137:39]: mapping [block]
          "Type" [plain @138:5..138:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @138:11..138:48]
          "Properties" [plain @139:5..139:15]: mapping [block]
            "SubnetId" [plain @140:7..140:15]: "PublicSubnetTwo" [plain @140:22..140:37] !<!Ref>
            "RouteTableId" [plain @141:7..141:19]: "PublicRouteTable" [plain @141:26..141:42] !<!Ref>
        "NatGatewayOneAttachment" [plain @147:3..147:26]: mapping [block]
          "Type" [plain @148:5..148:9]: "AWS::EC2::EIP" [plain @148:11..148:24]
          "DependsOn" [plain @149:5..149:14]: "GatewayAttachement" [plain @149:16..149:34]
          "Properties" [plain @150:5..150:15]: mapping [block]
            "Domain" [plain @151:7..151:13]: "vpc" [plain @151:15..151:18]
        "NatGatewayTwoAttachment" [plain @153:3..153:26]: mapping [block]
          "Type" [plain @154:5..154:9]: "AWS::EC2::EIP" [plain @154:11..154:24]
          "DependsOn" [plain @155:5..155:14]: "GatewayAttachement" [plain @155:16..155:34]
          "Properties" [plain @156:5..156:15]: mapping [block]
            "Domain" [plain @157:7..157:13]: "vpc" [plain @157:15..157:18]
        "NatGatewayOne" [plain @159:3..159:16]: mapping [block]
          "Type" [plain @160:5..160:9]: "AWS::EC2::NatGateway" [plain @160:11..160:31]
          "Properties" [plain @161:5..161:15]: mapping [block]
            "AllocationId" [plain @162:7..162:19]: "NatGatewayOneAttachment.AllocationId" [plain @162:29..162:65] !<!GetAtt>
            "SubnetId" [plain @163:7..163:15]: "PublicSubnetOne" [plain @163:22..163:37] !<!Ref>
        "NatGatewayTwo" [plain @165:3..165:16]: mapping [block]
          "Type" [plain @166:5..166:9]: "AWS::EC2::NatGateway" [plain @166:11..166:31]
          "Properties" [plain @167:5..167:15]: mapping [block]
            "AllocationId" [plain @168:7..168:19]: "NatGatewayTwoAttachment.AllocationId" [plain @168:29..168:65] !<!GetAtt>
            "SubnetId" [plain @169:7..169:15]: "PublicSubnetTwo" [plain @169:22..169:37] !<!Ref>
        "PrivateRouteTableOne" [plain @171:3..171:23]: mapping [block]
          "Type" [plain @172:5..172:9]: "AWS::EC2::RouteTable" [plain @172:11..172:31]
          "Properties" [plain @173:5..173:15]: mapping [block]
            "VpcId" [plain @174:7..174:12]: "VPC" [plain @174:19..174:22] !<!Ref>
        "PrivateRouteOne" [plain @176:3..176:18]: mapping [block]
          "Type" [plain @177:5..177:9]: "AWS::EC2::Route" [plain @177:11..177:26]
          "Properties" [plain @178:5..178:15]: mapping [block]
            "RouteTableId" [plain @179:7..179:19]: "PrivateRouteTableOne" [plain @179:26..179:46] !<!Ref>
            "DestinationCidrBlock" [plain @180:7..180:27]: "0.0.0.0/0" [plain @180:29..180:38]
            "NatGatewayId" [plain @181:7..181:19]: "NatGatewayOne" [plain @181:26..181:39] !<!Ref>
        "PrivateRouteTableOneAssociation" [plain @183:3..183:34]: mapping [block]
          "Type" [plain @184:5..184:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @184:11..184:48]
          "Properties" [plain @185:5..185:15]: mapping [block]
            "RouteTableId" [plain @186:7..186:19]: "PrivateRouteTableOne" [plain @186:26..186:46] !<!Ref>
            "SubnetId" [plain @187:7..187:15]: "PrivateSubnetOne" [plain @187:22..187:38] !<!Ref>
        "PrivateRouteTableTwo" [plain @189:3..189:23]: mapping [block]
          "Type" [plain @190:5..190:9]: "AWS::EC2::RouteTable" [plain @190:11..190:31]
          "Properties" [plain @191:5..191:15]: mapping [block]
            "VpcId" [plain @192:7..192:12]: "VPC" [plain @192:19..192:22] !<!Ref>
        "PrivateRouteTwo" [plain @194:3..194:18]: mapping [block]
          "Type" [plain @195:5..195:9]: "AWS::EC2::Route" [plain @195:11..195:26]
          "Properties" [plain @196:5..196:15]: mapping [block]
            "RouteTableId" [plain @197:7..197:19]: "PrivateRouteTableTwo" [plain @197:26..197:46] !<!Ref>
            "DestinationCidrBlock" [plain @198:7..198:27]: "0.0.0.0/0" [plain @198:29..198:38]
            "NatGatewayId" [plain @199:7..199:19]: "NatGatewayTwo" [plain @199:26..199:39] !<!Ref>
        "PrivateRouteTableTwoAssociation" [plain @201:3..201:34]: mapping [block]
          "Type" [plain @202:5..202:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @202:11..202:48]
          "Properties" [plain @203:5..203:15]: mapping [block]
            "RouteTableId" [plain @204:7..204:19]: "PrivateRouteTableTwo" [plain @204:26..204:46] !<!Ref>
            "SubnetId" [plain @205:7..205:15]: "PrivateSubnetTwo" [plain @205:22..205:38] !<!Ref>
        "DynamoDBEndpoint" [plain @212:3..212:19]: mapping [block]
          "Type" [plain @213:5..213:9]: "AWS::EC2::VPCEndpoint" [plain @213:11..213:32]
          "Properties" [plain @214:5..214:15]: mapping [block]
            "PolicyDocument" [plain @215:7..215:21]: mapping [block]
              "Version" [plain @216:9..216:16]: "2012-10-17" [double-quoted @216:18..216:30]
              "Statement" [plain @217:9..217:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @218:13..218:19]: "Allow" [plain @218:21..218:26]
                  "Action" [plain @219:13..219:19]: "*" [single-quoted @219:21..219:24]
                  "Principal" [plain @220:13..220:22]: "*" [single-quoted @220:24..220:27]
                  "Resource" [plain @221:13..221:21]: "*" [single-quoted @221:23..221:26]
            "RouteTableIds" [plain @222:7..222:20]: sequence [block]
              - "PrivateRouteTableOne" [plain @223:16..223:36] !<!Ref>
              - "PrivateRouteTableTwo" [plain @224:16..224:36] !<!Ref>
            "ServiceName" [plain @225:7..225:18]: sequence [block] !<!Join>
              - "" [double-quoted @226:11..226:13]
              - sequence [block]
                - "com.amazonaws." [plain @227:13..227:27]
                - "AWS::Region" [plain @228:18..228:29] !<!Ref>
                - ".dynamodb" [plain @229:13..229:22]
            "VpcId" [plain @230:7..230:12]: "VPC" [plain @230:19..230:22] !<!Ref>
        "ECSCluster" [plain @233:3..233:13]: mapping [block]
          "Type" [plain @234:5..234:9]: "AWS::ECS::Cluster" [plain @234:11..234:28]
        "FargateContainerSecurityGroup" [plain @242:3..242:32]: mapping [block]
          "Type" [plain @243:5..243:9]: "AWS::EC2::SecurityGroup" [plain @243:11..243:34]
          "Properties" [plain @244:5..244:15]: mapping [block]
            "GroupDescription" [plain @245:7..245:23]: "Access to the Fargate containers" [plain @245:25..245:57]
            "VpcId" [plain @246:7..246:12]: "VPC" [plain @246:19..246:22] !<!Ref>
        "EcsSecurityGroupIngressFromPublicALB" [plain @248:3..248:39]: mapping [block]
          "Type" [plain @249:5..249:9]: "AWS::EC2::SecurityGroupIngress" [plain @249:11..249:41]
          "Properties" [plain @250:5..250:15]: mapping [block]
            "Description" [plain @251:7..251:18]: "Ingress from the public ALB" [plain @251:20..251:47]
            "GroupId" [plain @252:7..252:14]: "FargateContainerSecurityGroup" [plain @252:21..252:50] !<!Ref>
            "IpProtocol" [plain @253:7..253:17]: "-1" [double-quoted @253:19..253:23]
            "SourceSecurityGroupId" [plain @254:7..254:28]: "PublicLoadBalancerSG" [plain @254:35..254:55] !<!Ref>
        "EcsSecurityGroupIngressFromPrivateALB" [plain @256:3..256:40]: mapping [block]
          "Type" [plain @257:5..257:9]: "AWS::EC2::SecurityGroupIngress" [plain @257:11..257:41]
          "Properties" [plain @258:5..258:15]: mapping [block]
            "Description" [plain @259:7..259:18]: "Ingress from the private ALB" [plain @259:20..259:48]
            "GroupId" [plain @260:7..260:14]: "FargateContainerSecurityGroup" [plain @260:21..260:50] !<!Ref>
            "IpProtocol" [plain @261:7..261:17]: "-1" [double-quoted @261:19..261:23]
            "SourceSecurityGroupId" [plain @262:7..262:28]: "PrivateLoadBalancerSG" [plain @262:35..262:56] !<!Ref>
        "EcsSecurityGroupIngressFromSelf" [plain @264:3..264:34]: mapping [block]
          "Type" [plain @265:5..265:9]: "AWS::EC2::SecurityGroupIngress" [plain @265:11..265:41]
          "Properties" [plain @266:5..266:15]: mapping [block]
            "Description" [plain @267:7..267:18]: "Ingress from other containers in the same security group" [plain @267:20..267:76]
            "GroupId" [plain @268:7..268:14]: "FargateContainerSecurityGroup" [plain @268:21..268:50] !<!Ref>
            "IpProtocol" [plain @269:7..269:17]: "-1" [double-quoted @269:19..269:23]
            "SourceSecurityGroupId" [plain @270:7..270:28]: "FargateContainerSecurityGroup" [plain @270:35..270:64] !<!Ref>
        "PublicLoadBalancerSG" [plain @285:3..285:23]: mapping [block]
          "Type" [plain @286:5..286:9]: "AWS::EC2::SecurityGroup" [plain @286:11..286:34]
          "Properties" [plain @287:5..287:15]: mapping [block]
            "GroupDescription" [plain @288:7..288:23]: "Access to the public facing load balancer" [plain @288:25..288:66]
            "VpcId" [plain @289:7..289:12]: "VPC" [plain @289:19..289:22] !<!Ref>
            "SecurityGroupIngress" [plain @290:7..290:27]: sequence [block]
              - mapping [block]
                "CidrIp" [plain @293:11..293:17]: "0.0.0.0/0" [plain @293:19..293:28]
                "IpProtocol" [plain @294:11..294:21]: "-1" [double-quoted @294:23..294:27]
        "PublicLoadBalancer" [plain @296:3..296:21]: mapping [block]
          "Type" [plain @297:5..297:9]: "AWS::ElasticLoadBalancingV2::LoadBalancer" [plain @297:11..297:52]
          "DependsOn" [plain @298:5..298:14]: "GatewayAttachement" [plain @298:16..298:34]
          "Properties" [plain @299:5..299:15]: mapping [block]
            "Scheme" [plain @300:7..300:13]: "internet-facing" [plain @300:15..300:30]
            "LoadBalancerAttributes" [plain @301:7..301:29]: sequence [block]
              - mapping [block]
                "Key" [plain @302:11..302:14]: "idle_timeout.timeout_seconds" [plain @302:16..302:44]
                "Value" [plain @303:11..303:16]: "30" [double-quoted @303:18..303:22]
            "Subnets" [plain @304:7..304:14]: sequence [block]
              - "PublicSubnetOne" [plain @308:16..308:31] !<!Ref>
              - "PublicSubnetTwo" [plain @309:16..309:31] !<!Ref>
            "SecurityGroups" [plain @310:7..310:21]: sequence [block]
              - "PublicLoadBalancerSG" [plain @311:16..311:36] !<!Ref>
        "DummyTargetGroupPublic" [plain @315:3..315:25]: mapping [block]
          "Type" [plain @316:5..316:9]: "AWS::ElasticLoadBalancingV2::TargetGroup" [plain @316:11..316:51]
          "Properties" [plain @317:5..317:15]: mapping [block]
            "HealthCheckIntervalSeconds" [plain @318:7..318:33]: "6" [plain @318:35..318:36]
            "HealthCheckPath" [plain @319:7..319:22]: "/" [plain @319:24..319:25]
            "HealthCheckProtocol" [plain @320:7..320:26]: "HTTP" [plain @320:28..320:32]
            "HealthCheckTimeoutSeconds" [plain @321:7..321:32]: "5" [plain @321:34..321:35]
            "HealthyThresholdCount" [plain @322:7..322:28]: "2" [plain @322:30..322:31]
            "Name" [plain @323:7..323:11]: sequence [block] !<!Join>
              - "-" [single-quoted @324:11..324:14]
              - sequence [block]
                - "AWS::StackName" [plain @325:18..325:32] !<!Ref>
                - "drop-1" [plain @326:13..326:19]
            "Port" [plain @327:7..327:11]: "80" [plain @327:13..327:15]
            "Protocol" [plain @328:7..328:15]: "HTTP" [plain @328:17..328:21]
            "UnhealthyThresholdCount" [plain @329:7..329:30]: "2" [plain @329:32..329:33]
            "VpcId" [plain @330:7..330:12]: "VPC" [plain @330:19..330:22] !<!Ref>
        "PublicLoadBalancerListener" [plain @332:3..332:29]: mapping [block]
          "Type" [plain @333:5..333:9]: "AWS::ElasticLoadBalancingV2::Listener" [plain @333:11..333:48]
          "Properties" [plain @334:5..334:15]: mapping [block]
            "DefaultActions" [plain @335:7..335:21]: sequence [block]
              - mapping [block]
                "TargetGroupArn" [plain @336:11..336:25]: "DummyTargetGroupPublic" [plain @336:32..336:54] !<!Ref>
                "Type" [plain @337:11..337:15]: "forward" [plain @337:17..337:24]
            "LoadBalancerArn" [plain @338:7..338:22]: "PublicLoadBalancer" [plain @338:29..338:47] !<!Ref>
            "Port" [plain @339:7..339:11]: "80" [plain @339:13..339:15]
            "Protocol" [plain @340:7..340:15]: "HTTP" [plain @340:17..340:21]
        "PrivateLoadBalancerSG" [plain @345:3..345:24]: mapping [block]
          "Type" [plain @346:5..346:9]: "AWS::EC2::SecurityGroup" [plain @346:11..346:34]
          "Properties" [plain @347:5..347:15]: mapping [block]
            "GroupDescription" [plain @348:7..348:23]: "Access to the internal load balancer" [plain @348:25..348:61]
            "VpcId" [plain @349:7..349:12]: "VPC" [plain @349:19..349:22] !<!Ref>
        "PrivateLoadBalancerIngressFromECS" [plain @351:3..351:36]: mapping [block]
          "Type" [plain @352:5..352:9]: "AWS::EC2::SecurityGroupIngress" [plain @352:11..352:41]
          "Properties" [plain @353:5..353:15]: mapping [block]
            "Description" [plain @354:7..354:18]: "Only accept traffic from a container in the fargate container security group" [plain @354:20..354:96]
            "GroupId" [plain @355:7..355:14]: "PrivateLoadBalancerSG" [plain @355:21..355:42] !<!Ref>
            "IpProtocol" [plain @356:7..356:17]: "-1" [double-quoted @356:19..356:23]
            "SourceSecurityGroupId" [plain @357:7..357:28]: "FargateContainerSecurityGroup" [plain @357:35..357:64] !<!Ref>
        "PrivateLoadBalancer" [plain @359:3..359:22]: mapping [block]
          "Type" [plain @360:5..360:9]: "AWS::ElasticLoadBalancingV2::LoadBalancer" [plain @360:11..360:52]
          "Properties" [plain @361:5..361:15]: mapping [block]
            "Scheme" [plain @362:7..362:13]: "internal" [plain @362:15..362:23]
            "LoadBalancerAttributes" [plain @363:7..363:29]: sequence [block]
              - mapping [block]
                "Key" [plain @364:11..364:14]: "idle_timeout.timeout_seconds" [plain @364:16..364:44]
                "Value" [plain @365:11..365:16]: "30" [double-quoted @365:18..365:22]
            "Subnets" [plain @366:7..366:14]: sequence [block]
              - "PrivateSubnetOne" [plain @370:16..370:32] !<!Ref>
              - "PrivateSubnetTwo" [plain @371:16..371:32] !<!Ref>
            "SecurityGroups" [plain @372:7..372:21]: sequence [block]
              - "PrivateLoadBalancerSG" [plain @373:16..373:37] !<!Ref>
        "DummyTargetGroupPrivate" [plain @377:3..377:26]: mapping [block]
          "Type" [plain @378:5..378:9]: "AWS::ElasticLoadBalancingV2::TargetGroup" [plain @378:11..378:51]
          "Properties" [plain @379:5..379:15]: mapping [block]
            "HealthCheckIntervalSeconds" [plain @380:7..380:33]: "6" [plain @380:35..380:36]
            "HealthCheckPath" [plain @381:7..381:22]: "/" [plain @381:24..381:25]
            "HealthCheckProtocol" [plain @382:7..382:26]: "HTTP" [plain @382:28..382:32]
            "HealthCheckTimeoutSeconds" [plain @383:7..383:32]: "5" [plain @383:34..383:35]
            "HealthyThresholdCount" [plain @384:7..384:28]: "2" [plain @384:30..384:31]
            "Name" [plain @385:7..385:11]: sequence [block] !<!Join>
              - "-" [single-quoted @386:11..386:14]
              - sequence [block]
                - "AWS::StackName" [plain @387:18..387:32] !<!Ref>
                - "drop-2" [plain @388:13..388:19]
            "Port" [plain @389:7..389:11]: "80" [plain @389:13..389:15]
            "Protocol" [plain @390:7..390:15]: "HTTP" [plain @390:17..390:21]
            "UnhealthyThresholdCount" [plain @391:7..391:30]: "2" [plain @391:32..391:33]
            "VpcId" [plain @392:7..392:12]: "VPC" [plain @392:19..392:22] !<!Ref>
        "PrivateLoadBalancerListener" [plain @394:3..394:30]: mapping [block]
          "Type" [plain @395:5..395:9]: "AWS::ElasticLoadBalancingV2::Listener" [plain @395:11..395:48]
          "Properties" [plain @396:5..396:15]: mapping [block]
            "DefaultActions" [plain @397:7..397:21]: sequence [block]
              - mapping [block]
                "TargetGroupArn" [plain @398:11..398:25]: "DummyTargetGroupPrivate" [plain @398:32..398:55] !<!Ref>
                "Type" [plain @399:11..399:15]: "forward" [plain @399:17..399:24]
            "LoadBalancerArn" [plain @400:7..400:22]: "PrivateLoadBalancer" [plain @400:29..400:48] !<!Ref>
            "Port" [plain @401:7..401:11]: "80" [plain @401:13..401:15]
            "Protocol" [plain @402:7..402:15]: "HTTP" [plain @402:17..402:21]
        "ECSRole" [plain @408:3..408:10]: mapping [block]
          "Type" [plain @409:5..409:9]: "AWS::IAM::Role" [plain @409:11..409:25]
          "Properties" [plain @410:5..410:15]: mapping [block]
            "AssumeRolePolicyDocument" [plain @411:7..411:31]: mapping [block]
              "Statement" [plain @412:9..412:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @413:13..413:19]: "Allow" [plain @413:21..413:26]
                  "Principal" [plain @414:13..414:22]: mapping [block]
                    "Service" [plain @415:15..415:22]: sequence [block]
                      - "ecs.amazonaws.com" [plain @416:19..416:36]
                  "Action" [plain @417:13..417:19]: sequence [block]
                    - "sts:AssumeRole" [plain @418:17..418:31]
            "Path" [plain @419:7..419:11]: "/" [plain @419:13..419:14]
            "Policies" [plain @420:7..420:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @421:11..421:21]: "ecs-service" [plain @421:23..421:34]
                "PolicyDocument" [plain @422:11..422:25]: mapping [block]
                  "Statement" [plain @423:13..423:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @424:17..424:23]: "Allow" [plain @424:25..424:30]
                      "Action" [plain @425:17..425:23]: sequence [block]
                        - "ec2:AttachNetworkInterface" [plain @429:21..429:47]
                        - "ec2:CreateNetworkInterface" [plain @430:21..430:47]
                        - "ec2:CreateNetworkInterfacePermission" [plain @431:21..431:57]
                        - "ec2:DeleteNetworkInterface" [plain @432:21..432:47]
                        - "ec2:DeleteNetworkInterfacePermission" [plain @433:21..433:57]
                        - "ec2:Describe*" [plain @434:21..434:34]
                        - "ec2:DetachNetworkInterface" [plain @435:21..435:47]
                        - "elasticloadbalancing:DeregisterInstancesFromLoadBalancer" [plain @439:21..439:77]
                        - "elasticloadbalancing:DeregisterTargets" [plain @440:21..440:59]
                        - "elasticloadbalancing:Describe*" [plain @441:21..441:51]
                        - "elasticloadbalancing:RegisterInstancesWithLoadBalancer" [plain @442:21..442:75]
                        - "elasticloadbalancing:RegisterTargets" [plain @443:21..443:57]
                      "Resource" [plain @444:17..444:25]: "*" [single-quoted @444:27..444:30]
        "ECSTaskExecutionRole" [plain @447:3..447:23]: mapping [block]
          "Type" [plain @448:5..448:9]: "AWS::IAM::Role" [plain @448:11..448:25]
          "Properties" [plain @449:5..449:15]: mapping [block]
            "AssumeRolePolicyDocument" [plain @450:7..450:31]: mapping [block]
              "Statement" [plain @451:9..451:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @452:13..452:19]: "Allow" [plain @452:21..452:26]
                  "Principal" [plain @453:13..453:22]: mapping [block]
                    "Service" [plain @454:15..454:22]: sequence [block]
                      - "ecs-tasks.amazonaws.com" [plain @455:19..455:42]
                  "Action" [plain @456:13..456:19]: sequence [block]
                    - "sts:AssumeRole" [plain @457:17..457:31]
            "Path" [plain @458:7..458:11]: "/" [plain @458:13..458:14]
            "Policies" [plain @459:7..459:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @460:11..460:21]: "AmazonECSTaskExecutionRolePolicy" [plain @460:23..460:55]
                "PolicyDocument" [plain @461:11..461:25]: mapping [block]
                  "Statement" [plain @462:13..462:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @463:17..463:23]: "Allow" [plain @463:25..463:30]
                      "Action" [plain @464:17..464:23]: sequence [block]
                        - "ecr:GetAuthorizationToken" [plain @467:21..467:46]
                        - "ecr:BatchCheckLayerAvailability" [plain @468:21..468:52]
                        - "ecr:GetDownloadUrlForLayer" [plain @469:21..469:47]
                        - "ecr:BatchGetImage" [plain @470:21..470:38]
                        - "logs:CreateLogStream" [plain @473:21..473:41]
                        - "logs:PutLogEvents" [plain @474:21..474:38]
                      "Resource" [plain @475:17..475:25]: "*" [single-quoted @475:27..475:30]
      "Outputs" [plain @480:1..480:8]: mapping [block]
        "ClusterName" [plain @481:3..481:14]: mapping [block]
          "Description" [plain @482:5..482:16]: "The name of the ECS cluster" [plain @482:18..482:45]
          "Value" [plain @483:5..483:10]: "ECSCluster" [plain @483:17..483:27] !<!Ref>
          "Export" [plain @484:5..484:11]: mapping [block]
            "Name" [plain @485:7..485:11]: sequence [block] !<!Join>
              - ":" [single-quoted @486:11..486:14]
              - sequence [block]
                - "AWS::StackName" [plain @487:18..487:32] !<!Ref>
                - "ClusterName" [plain @488:13..488:24]
        "InternalUrl" [plain @490:3..490:14]: mapping [block]
          "Description" [plain @491:5..491:16]: "The url of the internal load balancer" [plain @491:18..491:55]
          "Value" [plain @492:5..492:10]: sequence [block] !<!Join>
            - "" [double-quoted @493:9..493:11]
            - sequence [block]
              - "http://" [plain @494:11..494:18]
              - "PrivateLoadBalancer.DNSName" [plain @495:19..495:46] !<!GetAtt>
          "Export" [plain @496:5..496:11]: mapping [block]
            "Name" [plain @497:7..497:11]: sequence [block] !<!Join>
              - ":" [single-quoted @498:11..498:14]
              - sequence [block]
                - "AWS::StackName" [plain @499:18..499:32] !<!Ref>
                - "InternalUrl" [plain @500:13..500:24]
        "ExternalUrl" [plain @502:3..502:14]: mapping [block]
          "Description" [plain @503:5..503:16]: "The url of the external load balancer" [plain @503:18..503:55]
          "Value" [plain @504:5..504:10]: sequence [block] !<!Join>
            - "" [double-quoted @505:9..505:11]
            - sequence [block]
              - "http://" [plain @506:11..506:18]
              - "PublicLoadBalancer.DNSName" [plain @507:19..507:45] !<!GetAtt>
          "Export" [plain @508:5..508:11]: mapping [block]
            "Name" [plain @509:7..509:11]: sequence [block] !<!Join>
              - ":" [single-quoted @510:11..510:14]
              - sequence [block]
                - "AWS::StackName" [plain @511:18..511:32] !<!Ref>
                - "ExternalUrl" [plain @512:13..512:24]
        "ECSRole" [plain @514:3..514:10]: mapping [block]
          "Description" [plain @515:5..515:16]: "The ARN of the ECS role" [plain @515:18..515:41]
          "Value" [plain @516:5..516:10]: "ECSRole.Arn" [plain @516:20..516:31] !<!GetAtt>
          "Export" [plain @517:5..517:11]: mapping [block]
            "Name" [plain @518:7..518:11]: sequence [block] !<!Join>
              - ":" [single-quoted @519:11..519:14]
              - sequence [block]
                - "AWS::StackName" [plain @520:18..520:32] !<!Ref>
                - "ECSRole" [plain @521:13..521:20]
        "ECSTaskExecutionRole" [plain @523:3..523:23]: mapping [block]
          "Description" [plain @524:5..524:16]: "The ARN of the ECS role" [plain @524:18..524:41]
          "Value" [plain @525:5..525:10]: "ECSTaskExecutionRole.Arn" [plain @525:20..525:44] !<!GetAtt>
          "Export" [plain @526:5..526:11]: mapping [block]
            "Name" [plain @527:7..527:11]: sequence [block] !<!Join>
              - ":" [single-quoted @528:11..528:14]
              - sequence [block]
                - "AWS::StackName" [plain @529:18..529:32] !<!Ref>
                - "ECSTaskExecutionRole" [plain @530:13..530:33]
        "PublicListener" [plain @532:3..532:17]: mapping [block]
          "Description" [plain @533:5..533:16]: "The ARN of the public load balancer's Listener" [plain @533:18..533:64]
          "Value" [plain @534:5..534:10]: "PublicLoadBalancerListener" [plain @534:17..534:43] !<!Ref>
          "Export" [plain @535:5..535:11]: mapping [block]
            "Name" [plain @536:7..536:11]: sequence [block] !<!Join>
              - ":" [single-quoted @537:11..537:14]
              - sequence [block]
                - "AWS::StackName" [plain @538:18..538:32] !<!Ref>
                - "PublicListener" [plain @539:13..539:27]
        "PrivateListener" [plain @541:3..541:18]: mapping [block]
          "Description" [plain @542:5..542:16]: "The ARN of the private load balancer's Listener" [plain @542:18..542:65]
          "Value" [plain @543:5..543:10]: "PrivateLoadBalancerListener" [plain @543:17..543:44] !<!Ref>
          "Export" [plain @544:5..544:11]: mapping [block]
            "Name" [plain @545:7..545:11]: sequence [block] !<!Join>
              - ":" [single-quoted @546:11..546:14]
              - sequence [block]
                - "AWS::StackName" [plain @547:18..547:32] !<!Ref>
                - "PrivateListener" [plain @548:13..548:28]
        "VPCId" [plain @550:3..550:8]: mapping [block]
          "Description" [plain @551:5..551:16]: "The ID of the VPC that this stack is deployed in" [plain @551:18..551:66]
          "Value" [plain @552:5..552:10]: "VPC" [plain @552:17..552:20] !<!Ref>
          "Export" [plain @553:5..553:11]: mapping [block]
            "Name" [plain @554:7..554:11]: sequence [block] !<!Join>
              - ":" [single-quoted @555:11..555:14]
              - sequence [block]
                - "AWS::StackName" [plain @556:18..556:32] !<!Ref>
                - "VPCId" [plain @557:13..557:18]
        "PublicSubnetOne" [plain @559:3..559:18]: mapping [block]
          "Description" [plain @560:5..560:16]: "Public subnet one" [plain @560:18..560:35]
          "Value" [plain @561:5..561:10]: "PublicSubnetOne" [plain @561:17..561:32] !<!Ref>
          "Export" [plain @562:5..562:11]: mapping [block]
            "Name" [plain @563:7..563:11]: sequence [block] !<!Join>
              - ":" [single-quoted @564:11..564:14]
              - sequence [block]
                - "AWS::StackName" [plain @565:18..565:32] !<!Ref>
                - "PublicSubnetOne" [plain @566:13..566:28]
        "PublicSubnetTwo" [plain @568:3..568:18]: mapping [block]
          "Description" [plain @569:5..569:16]: "Public subnet two" [plain @569:18..569:35]
          "Value" [plain @570:5..570:10]: "PublicSubnetTwo" [plain @570:17..570:32] !<!Ref>
          "Export" [plain @571:5..571:11]: mapping [block]
            "Name" [plain @572:7..572:11]: sequence [block] !<!Join>
              - ":" [single-quoted @573:11..573:14]
              - sequence [block]
                - "AWS::StackName" [plain @574:18..574:32] !<!Ref>
                - "PublicSubnetTwo" [plain @575:13..575:28]
        "PrivateSubnetOne" [plain @577:3..577:19]: mapping [block]
          "Description" [plain @578:5..578:16]: "Private subnet one" [plain @578:18..578:36]
          "Value" [plain @579:5..579:10]: "PrivateSubnetOne" [plain @579:17..579:33] !<!Ref>
          "Export" [plain @580:5..580:11]: mapping [block]
            "Name" [plain @581:7..581:11]: sequence [block] !<!Join>
              - ":" [single-quoted @582:11..582:14]
              - sequence [block]
                - "AWS::StackName" [plain @583:18..583:32] !<!Ref>
                - "PrivateSubnetOne" [plain @584:13..584:29]
        "PrivateSubnetTwo" [plain @586:3..586:19]: mapping [block]
          "Description" [plain @587:5..587:16]: "Private subnet two" [plain @587:18..587:36]
          "Value" [plain @588:5..588:10]: "PrivateSubnetTwo" [plain @588:17..588:33] !<!Ref>
          "Export" [plain @589:5..589:11]: mapping [block]
            "Name" [plain @590:7..590:11]: sequence [block] !<!Join>
              - ":" [single-quoted @591:11..591:14]
              - sequence [block]
                - "AWS::StackName" [plain @592:18..592:32] !<!Ref>
                - "PrivateSubnetTwo" [plain @593:13..593:29]
        "FargateContainerSecurityGroup" [plain @595:3..595:32]: mapping [block]
          "Description" [plain @596:5..596:16]: "A security group used to allow Fargate containers to receive traffic" [plain @596:18..596:86]
          "Value" [plain @597:5..597:10]: "FargateContainerSecurityGroup" [plain @597:17..597:46] !<!Ref>
          "Export" [plain @598:5..598:11]: mapping [block]
            "Name" [plain @599:7..599:11]: sequence [block] !<!Join>
              - ":" [single-quoted @600:11..600:14]
              - sequence [block]
                - "AWS::StackName" [plain @601:18..601:32] !<!Ref>
                - "FargateContainerSecurityGroup" [plain @602:13..602:42]
    "#);
}
