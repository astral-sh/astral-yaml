#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-elasticache-elasticache-snapshot-06d0965d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Create a VPC containing two subnets and an auto scaling group containing instances with Internet access." [plain @3:14..3:118]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "RedisNodeType" [plain @6:3..6:16]: mapping [block]
          "Description" [plain @7:5..7:16]: "elasticache Redis Node Instance Type" [plain @7:18..7:54]
          "Type" [plain @8:5..8:9]: "String" [plain @8:11..8:17]
          "AllowedValues" [plain @9:5..9:18]: sequence [block]
            - "cache.m3.medium" [plain @10:9..10:24]
          "Default" [plain @11:5..11:12]: "cache.m3.medium" [plain @11:14..11:29]
          "ConstraintDescription" [plain @12:5..12:26]: "must be an m3.medium - the least costly machine that can use a Replication Group." [plain @12:28..12:109]
        "EnableSnapshotting" [plain @14:3..14:21]: mapping [block]
          "Description" [plain @15:5..15:16]: "elasticache snapshot enable" [plain @15:18..15:45]
          "Type" [plain @16:5..16:9]: "String" [plain @16:11..16:17]
          "AllowedValues" [plain @17:5..17:18]: sequence [block]
            - "True" [double-quoted @18:9..18:15]
            - "False" [double-quoted @19:9..19:16]
          "Default" [plain @20:5..20:12]: "True" [double-quoted @20:14..20:20]
        "SnapshotRetentionLimit" [plain @22:3..22:25]: mapping [block]
          "Description" [plain @23:5..23:16]: "elasticache Snapshot Retention Limit" [plain @23:18..23:54]
          "Type" [plain @24:5..24:9]: "String" [plain @24:11..24:17]
          "Default" [plain @25:5..25:12]: "28" [double-quoted @25:14..25:18]
        "SnapshotWindow" [plain @27:3..27:17]: mapping [block]
          "Description" [plain @28:5..28:16]: "Snapshot Window" [plain @28:18..28:33]
          "Type" [plain @29:5..29:9]: "String" [plain @29:11..29:17]
          "Default" [plain @30:5..30:12]: "02:00-03:00" [plain @30:14..30:25]
      "Mappings" [plain @32:1..32:9]: mapping [block]
        "AWSRegion2AZ" [plain @33:3..33:15]: mapping [block]
          "us-east-1" [plain @34:5..34:14]: mapping [block]
            "A" [plain @35:7..35:8]: "us-east-1b" [plain @35:10..35:20]
            "B" [plain @36:7..36:8]: "us-east-1c" [plain @36:10..36:20]
            "C" [plain @37:7..37:8]: "us-east-1c" [plain @37:10..37:20]
            "D" [plain @38:7..38:8]: "us-east-1d" [plain @38:10..38:20]
          "us-west-1" [plain @39:5..39:14]: mapping [block]
            "A" [plain @40:7..40:8]: "us-west-1b" [plain @40:10..40:20]
            "B" [plain @41:7..41:8]: "us-west-1c" [plain @41:10..41:20]
          "us-west-2" [plain @42:5..42:14]: mapping [block]
            "A" [plain @43:7..43:8]: "us-west-2a" [plain @43:10..43:20]
            "B" [plain @44:7..44:8]: "us-west-2b" [plain @44:10..44:20]
            "C" [plain @45:7..45:8]: "us-west-2c" [plain @45:10..45:20]
      "Conditions" [plain @47:1..47:11]: mapping [block]
        "EnableBackups" [plain @48:3..48:16]: sequence [block] !<!Equals>
          - "EnableSnapshotting" [plain @49:12..49:30] !<!Ref>
          - "True" [double-quoted @50:7..50:13]
      "Resources" [plain @52:1..52:10]: mapping [block]
        "VPC" [plain @53:3..53:6]: mapping [block]
          "Type" [plain @54:5..54:9]: "AWS::EC2::VPC" [plain @54:11..54:24]
          "Properties" [plain @55:5..55:15]: mapping [block]
            "CidrBlock" [plain @56:7..56:16]: "10.0.0.0/24" [plain @56:18..56:29]
        "InternetGateway" [plain @58:3..58:18]: mapping [block]
          "Type" [plain @59:5..59:9]: "AWS::EC2::InternetGateway" [plain @59:11..59:36]
        "PublicInternetRoute" [plain @61:3..61:22]: mapping [block]
          "Type" [plain @62:5..62:9]: "AWS::EC2::Route" [plain @62:11..62:26]
          "DependsOn" [plain @63:5..63:14]: sequence [block]
            - "InternetGateway" [plain @64:9..64:24]
            - "PublicInternetRouteTable" [plain @65:9..65:33]
          "Properties" [plain @66:5..66:15]: mapping [block]
            "DestinationCidrBlock" [plain @67:7..67:27]: "0.0.0.0/0" [plain @67:29..67:38]
            "GatewayId" [plain @68:7..68:16]: "InternetGateway" [plain @68:23..68:38] !<!Ref>
            "RouteTableId" [plain @69:7..69:19]: "PublicInternetRouteTable" [plain @69:26..69:50] !<!Ref>
        "VPCGatewayAttachment" [plain @71:3..71:23]: mapping [block]
          "Type" [plain @72:5..72:9]: "AWS::EC2::VPCGatewayAttachment" [plain @72:11..72:41]
          "Properties" [plain @73:5..73:15]: mapping [block]
            "InternetGatewayId" [plain @74:7..74:24]: "InternetGateway" [plain @74:31..74:46] !<!Ref>
            "VpcId" [plain @75:7..75:12]: "VPC" [plain @75:19..75:22] !<!Ref>
        "PublicInternetRouteTable" [plain @77:3..77:27]: mapping [block]
          "Type" [plain @78:5..78:9]: "AWS::EC2::RouteTable" [plain @78:11..78:31]
          "Properties" [plain @79:5..79:15]: mapping [block]
            "VpcId" [plain @80:7..80:12]: "VPC" [plain @80:19..80:22] !<!Ref>
        "PublicSubnetA" [plain @82:3..82:16]: mapping [block]
          "Type" [plain @83:5..83:9]: "AWS::EC2::Subnet" [plain @83:11..83:27]
          "Properties" [plain @84:5..84:15]: mapping [block]
            "AvailabilityZone" [plain @85:7..85:23]: sequence [block] !<!FindInMap>
              - "AWSRegion2AZ" [plain @86:11..86:23]
              - "AWS::Region" [plain @87:16..87:27] !<!Ref>
              - "A" [plain @88:11..88:12]
            "CidrBlock" [plain @89:7..89:16]: "10.0.0.0/25" [plain @89:18..89:29]
            "VpcId" [plain @90:7..90:12]: "VPC" [plain @90:19..90:22] !<!Ref>
        "PublicSubnetB" [plain @92:3..92:16]: mapping [block]
          "Type" [plain @93:5..93:9]: "AWS::EC2::Subnet" [plain @93:11..93:27]
          "Properties" [plain @94:5..94:15]: mapping [block]
            "AvailabilityZone" [plain @95:7..95:23]: sequence [block] !<!FindInMap>
              - "AWSRegion2AZ" [plain @96:11..96:23]
              - "AWS::Region" [plain @97:16..97:27] !<!Ref>
              - "B" [plain @98:11..98:12]
            "CidrBlock" [plain @99:7..99:16]: "10.0.0.128/25" [plain @99:18..99:31]
            "VpcId" [plain @100:7..100:12]: "VPC" [plain @100:19..100:22] !<!Ref>
        "PublicSubnetARouteTableAssociation" [plain @102:3..102:37]: mapping [block]
          "Type" [plain @103:5..103:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @103:11..103:48]
          "Properties" [plain @104:5..104:15]: mapping [block]
            "RouteTableId" [plain @105:7..105:19]: "PublicInternetRouteTable" [plain @105:26..105:50] !<!Ref>
            "SubnetId" [plain @106:7..106:15]: "PublicSubnetA" [plain @106:22..106:35] !<!Ref>
        "PublicSubnetBRouteTableAssociation" [plain @108:3..108:37]: mapping [block]
          "Type" [plain @109:5..109:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @109:11..109:48]
          "Properties" [plain @110:5..110:15]: mapping [block]
            "RouteTableId" [plain @111:7..111:19]: "PublicInternetRouteTable" [plain @111:26..111:50] !<!Ref>
            "SubnetId" [plain @112:7..112:15]: "PublicSubnetB" [plain @112:22..112:35] !<!Ref>
        "RedisParameterGroup" [plain @114:3..114:22]: mapping [block]
          "Type" [plain @115:5..115:9]: "AWS::ElastiCache::ParameterGroup" [plain @115:11..115:43]
          "Properties" [plain @116:5..116:15]: mapping [block]
            "CacheParameterGroupFamily" [plain @117:7..117:32]: "redis2.8" [plain @117:34..117:42]
            "Description" [plain @118:7..118:18]: "RedisParameterGroup" [plain @118:20..118:39]
        "RedisSubnetGroup" [plain @120:3..120:19]: mapping [block]
          "Type" [plain @121:5..121:9]: "AWS::ElastiCache::SubnetGroup" [plain @121:11..121:40]
          "Properties" [plain @122:5..122:15]: mapping [block]
            "Description" [plain @123:7..123:18]: "RedisSubnetGroup" [plain @123:20..123:36]
            "SubnetIds" [plain @124:7..124:16]: sequence [block]
              - "PublicSubnetA" [plain @125:16..125:29] !<!Ref>
              - "PublicSubnetB" [plain @126:16..126:29] !<!Ref>
        "RedisSecurityGroup" [plain @128:3..128:21]: mapping [block]
          "Type" [plain @129:5..129:9]: "AWS::EC2::SecurityGroup" [plain @129:11..129:34]
          "Properties" [plain @130:5..130:15]: mapping [block]
            "GroupDescription" [plain @131:7..131:23]: "RedisSecurityGroup" [plain @131:25..131:43]
            "VpcId" [plain @132:7..132:12]: "VPC" [plain @132:19..132:22] !<!Ref>
            "SecurityGroupIngress" [plain @133:7..133:27]: sequence [block]
              - mapping [block]
                "IpProtocol" [plain @134:11..134:21]: "tcp" [plain @134:23..134:26]
                "FromPort" [plain @135:11..135:19]: "6379" [double-quoted @135:21..135:27]
                "ToPort" [plain @136:11..136:17]: "6379" [double-quoted @136:19..136:25]
                "CidrIp" [plain @137:11..137:17]: "192.168.1.0/32" [plain @137:19..137:33]
        "RedisReplicationGroup" [plain @139:3..139:24]: mapping [block]
          "Type" [plain @140:5..140:9]: "AWS::ElastiCache::ReplicationGroup" [plain @140:11..140:45]
          "Properties" [plain @141:5..141:15]: mapping [block]
            "AutomaticFailoverEnabled" [plain @142:7..142:31]: "true" [double-quoted @142:33..142:39]
            "CacheNodeType" [plain @143:7..143:20]: "RedisNodeType" [plain @143:27..143:40] !<!Ref>
            "CacheParameterGroupName" [plain @144:7..144:30]: "RedisParameterGroup" [plain @144:37..144:56] !<!Ref>
            "CacheSubnetGroupName" [plain @145:7..145:27]: "RedisSubnetGroup" [plain @145:34..145:50] !<!Ref>
            "Engine" [plain @146:7..146:13]: "redis" [plain @146:15..146:20]
            "EngineVersion" [plain @147:7..147:20]: "2.8.24" [plain @147:22..147:28]
            "NumCacheClusters" [plain @148:7..148:23]: "2" [double-quoted @148:25..148:28]
            "PreferredCacheClusterAZs" [plain @149:7..149:31]: sequence [block]
              - sequence [block] !<!FindInMap>
                - "AWSRegion2AZ" [plain @151:13..151:25]
                - "AWS::Region" [plain @152:18..152:29] !<!Ref>
                - "A" [plain @153:13..153:14]
              - sequence [block] !<!FindInMap>
                - "AWSRegion2AZ" [plain @155:13..155:25]
                - "AWS::Region" [plain @156:18..156:29] !<!Ref>
                - "B" [plain @157:13..157:14]
            "ReplicationGroupDescription" [plain @158:7..158:34]: "RedisReplicationGroup" [plain @158:36..158:57]
            "SecurityGroupIds" [plain @159:7..159:23]: sequence [block]
              - "RedisSecurityGroup" [plain @160:16..160:34] !<!Ref>
        "IamRoleLambda" [plain @162:3..162:16]: mapping [block]
          "Type" [plain @163:5..163:9]: "AWS::IAM::Role" [plain @163:11..163:25]
          "Properties" [plain @164:5..164:15]: mapping [block]
            "AssumeRolePolicyDocument" [plain @165:7..165:31]: mapping [block]
              "Version" [plain @166:9..166:16]: "2012-10-17" [double-quoted @166:18..166:30]
              "Statement" [plain @167:9..167:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @168:13..168:19]: "Allow" [plain @168:21..168:26]
                  "Principal" [plain @169:13..169:22]: mapping [block]
                    "Service" [plain @170:15..170:22]: sequence [block]
                      - "lambda.amazonaws.com" [plain @171:19..171:39]
                  "Action" [plain @172:13..172:19]: sequence [block]
                    - "sts:AssumeRole" [plain @173:17..173:31]
            "ManagedPolicyArns" [plain @174:7..174:24]: sequence [block]
              - "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole" [plain @175:11..175:75]
            "Policies" [plain @176:7..176:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @177:11..177:21]: "ElastiCacheSnapshotPolicy" [plain @177:23..177:48]
                "PolicyDocument" [plain @178:11..178:25]: mapping [block]
                  "Version" [plain @179:13..179:20]: "2012-10-17" [double-quoted @179:22..179:34]
                  "Statement" [plain @180:13..180:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @181:17..181:23]: "Allow" [plain @181:25..181:30]
                      "Action" [plain @182:17..182:23]: sequence [block]
                        - "elasticache:ModifyReplicationGroup" [plain @183:21..183:55]
                      "Resource" [plain @184:17..184:25]: "RedisReplicationGroup" [plain @184:32..184:53] !<!Ref>
          "Condition" [plain @185:5..185:14]: "EnableBackups" [plain @185:16..185:29]
        "LambdaExecutePermission" [plain @187:3..187:26]: mapping [block]
          "Type" [plain @188:5..188:9]: "AWS::Lambda::Permission" [plain @188:11..188:34]
          "Properties" [plain @189:5..189:15]: mapping [block]
            "Action" [plain @190:7..190:13]: "lambda:InvokeFunction" [plain @190:15..190:36]
            "FunctionName" [plain @191:7..191:19]: "EnableShapshot.Arn" [plain @191:29..191:47] !<!GetAtt>
            "Principal" [plain @192:7..192:16]: "elasticache.amazonaws.com" [plain @192:18..192:43]
            "SourceAccount" [plain @193:7..193:20]: "${AWS::AccountId}" [plain @193:27..193:44] !<!Sub>
          "Condition" [plain @194:5..194:14]: "EnableBackups" [plain @194:16..194:29]
        "EnableShapshotTrigger" [plain @196:3..196:24]: mapping [block]
          "Type" [plain @197:5..197:9]: "Custom::Region" [plain @197:11..197:25]
          "DependsOn" [plain @198:5..198:14]: sequence [block]
            - "EnableShapshot" [plain @199:9..199:23]
            - "LambdaExecutePermission" [plain @200:9..200:32]
            - "RedisReplicationGroup" [plain @201:9..201:30]
          "Properties" [plain @202:5..202:15]: mapping [block]
            "ServiceToken" [plain @203:7..203:19]: "EnableShapshot.Arn" [plain @203:29..203:47] !<!GetAtt>
            "SSClusterId" [plain @204:7..204:18]: "RedisReplicationGroup" [plain @204:25..204:46] !<!Ref>
            "SSWindow" [plain @205:7..205:15]: "SnapshotWindow" [plain @205:22..205:36] !<!Ref>
            "SSRetentionLimit" [plain @206:7..206:23]: "SnapshotRetentionLimit" [plain @206:30..206:52] !<!Ref>
          "Condition" [plain @207:5..207:14]: "EnableBackups" [plain @207:16..207:29]
        "EnableShapshot" [plain @209:3..209:17]: mapping [block]
          "DeletionPolicy" [plain @210:5..210:19]: "Delete" [plain @210:21..210:27]
          "Type" [plain @211:5..211:9]: "AWS::Lambda::Function" [plain @211:11..211:32]
          "DependsOn" [plain @212:5..212:14]: sequence [block]
            - "IamRoleLambda" [plain @213:9..213:22]
          "Metadata" [plain @214:5..214:13]: mapping [block]
            "guard" [plain @215:7..215:12]: mapping [block]
              "SuppressedRules" [plain @216:9..216:24]: sequence [block]
                - "LAMBDA_INSIDE_VPC" [plain @217:13..217:30]
          "Properties" [plain @218:5..218:15]: mapping [block]
            "Code" [plain @219:7..219:11]: mapping [block]
              "ZipFile" [plain @220:9..220:16]: "elastic-code.js" [plain @220:31..220:46] !<!Rain::Embed>
            "Handler" [plain @221:7..221:14]: "index.handler" [plain @221:16..221:29]
            "MemorySize" [plain @222:7..222:17]: "128" [plain @222:19..222:22]
            "Role" [plain @223:7..223:11]: "IamRoleLambda.Arn" [plain @223:21..223:38] !<!GetAtt>
            "Runtime" [plain @224:7..224:14]: "nodejs20.x" [plain @224:16..224:26]
            "Timeout" [plain @225:7..225:14]: "30" [plain @225:16..225:18]
          "Condition" [plain @226:5..226:14]: "EnableBackups" [plain @226:16..226:29]
    "#);
}
