#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-ecs-fargatelaunchtype-services-private-subnet-public-service-491f11b9"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Deploy a service on AWS Fargate, hosted in a private subnet, but accessible via a public load balancer." [plain @3:14..3:117]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "StackName" [plain @6:3..6:12]: mapping [block]
          "Description" [plain @7:5..7:16]: "The name of the parent Fargate networking stack that you created. Necessary to locate and reference resources created by that stack." [plain @7:18..7:150]
          "Type" [plain @8:5..8:9]: "String" [plain @8:11..8:17]
          "Default" [plain @9:5..9:12]: "production" [plain @9:14..9:24]
        "ServiceName" [plain @11:3..11:14]: mapping [block]
          "Description" [plain @12:5..12:16]: "A name for the service" [plain @12:18..12:40]
          "Type" [plain @13:5..13:9]: "String" [plain @13:11..13:17]
          "Default" [plain @14:5..14:12]: "nginx" [plain @14:14..14:19]
        "ImageUrl" [plain @16:3..16:11]: mapping [block]
          "Description" [plain @17:5..17:16]: "The url of a docker image that contains the application process that will handle the traffic for this service" [plain @17:18..17:127]
          "Type" [plain @18:5..18:9]: "String" [plain @18:11..18:17]
          "Default" [plain @19:5..19:12]: "nginx" [plain @19:14..19:19]
        "ContainerPort" [plain @21:3..21:16]: mapping [block]
          "Description" [plain @22:5..22:16]: "What port number the application inside the docker container is binding to" [plain @22:18..22:92]
          "Type" [plain @23:5..23:9]: "Number" [plain @23:11..23:17]
          "Default" [plain @24:5..24:12]: "80" [plain @24:14..24:16]
        "ContainerCpu" [plain @26:3..26:15]: mapping [block]
          "Description" [plain @27:5..27:16]: "How much CPU to give the container. 1024 is 1 CPU" [plain @27:18..27:67]
          "Type" [plain @28:5..28:9]: "Number" [plain @28:11..28:17]
          "Default" [plain @29:5..29:12]: "256" [plain @29:14..29:17]
        "ContainerMemory" [plain @31:3..31:18]: mapping [block]
          "Description" [plain @32:5..32:16]: "How much memory in megabytes to give the container" [plain @32:18..32:68]
          "Type" [plain @33:5..33:9]: "Number" [plain @33:11..33:17]
          "Default" [plain @34:5..34:12]: "512" [plain @34:14..34:17]
        "Path" [plain @36:3..36:7]: mapping [block]
          "Description" [plain @37:5..37:16]: "A path on the public load balancer that this service should be connected to. Use * to send all load balancer traffic to this service." [plain @37:18..37:151]
          "Type" [plain @38:5..38:9]: "String" [plain @38:11..38:17]
          "Default" [plain @39:5..39:12]: "*" [single-quoted @39:14..39:17]
        "Priority" [plain @41:3..41:11]: mapping [block]
          "Description" [plain @42:5..42:16]: "The priority for the routing rule added to the load balancer. This only applies if your have multiple services which have been assigned to different paths on the load balancer." [plain @42:18..42:194]
          "Type" [plain @43:5..43:9]: "Number" [plain @43:11..43:17]
          "Default" [plain @44:5..44:12]: "1" [plain @44:14..44:15]
        "DesiredCount" [plain @46:3..46:15]: mapping [block]
          "Description" [plain @47:5..47:16]: "How many copies of the service task to run" [plain @47:18..47:60]
          "Type" [plain @48:5..48:9]: "Number" [plain @48:11..48:17]
          "Default" [plain @49:5..49:12]: "2" [plain @49:14..49:15]
        "Role" [plain @51:3..51:7]: mapping [block]
          "Description" [plain @52:5..52:16]: "(Optional) An IAM role to give the service's containers if the code within needs to access other AWS resources like S3 buckets, DynamoDB tables, etc" [plain @52:18..52:166]
          "Type" [plain @53:5..53:9]: "String" [plain @53:11..53:17]
          "Default" [plain @54:5..54:12]: "" [double-quoted @54:14..54:16]
      "Conditions" [plain @56:1..56:11]: mapping [block]
        "HasCustomRole" [plain @57:3..57:16]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "Role" [plain @59:14..59:18] !<!Ref>
            - "" [double-quoted @60:9..60:11]
      "Resources" [plain @62:1..62:10]: mapping [block]
        "TaskDefinition" [plain @66:3..66:17]: mapping [block]
          "Type" [plain @67:5..67:9]: "AWS::ECS::TaskDefinition" [plain @67:11..67:35]
          "Properties" [plain @68:5..68:15]: mapping [block]
            "Family" [plain @69:7..69:13]: "ServiceName" [plain @69:20..69:31] !<!Ref>
            "Cpu" [plain @70:7..70:10]: "ContainerCpu" [plain @70:17..70:29] !<!Ref>
            "Memory" [plain @71:7..71:13]: "ContainerMemory" [plain @71:20..71:35] !<!Ref>
            "NetworkMode" [plain @72:7..72:18]: "awsvpc" [plain @72:20..72:26]
            "RequiresCompatibilities" [plain @73:7..73:30]: sequence [block]
              - "FARGATE" [plain @74:11..74:18]
            "ExecutionRoleArn" [plain @75:7..75:23]: mapping [block] !<!ImportValue>
              "Fn::Join" [plain @76:9..76:17]: sequence [block]
                - ":" [single-quoted @77:13..77:16]
                - sequence [block]
                  - "StackName" [plain @78:20..78:29] !<!Ref>
                  - "ECSTaskExecutionRole" [plain @79:15..79:35]
            "TaskRoleArn" [plain @80:7..80:18]: sequence [block] !<!If>
              - "HasCustomRole" [plain @81:11..81:24]
              - "Role" [plain @82:16..82:20] !<!Ref>
              - "AWS::NoValue" [plain @83:16..83:28] !<!Ref>
            "ContainerDefinitions" [plain @84:7..84:27]: sequence [block]
              - mapping [block]
                "Name" [plain @85:11..85:15]: "ServiceName" [plain @85:22..85:33] !<!Ref>
                "Essential" [plain @86:11..86:20]: "true" [plain @86:22..86:26]
                "Cpu" [plain @87:11..87:14]: "ContainerCpu" [plain @87:21..87:33] !<!Ref>
                "Memory" [plain @88:11..88:17]: "ContainerMemory" [plain @88:24..88:39] !<!Ref>
                "Image" [plain @89:11..89:16]: "ImageUrl" [plain @89:23..89:31] !<!Ref>
                "PortMappings" [plain @90:11..90:23]: sequence [block]
                  - mapping [block]
                    "ContainerPort" [plain @91:15..91:28]: "ContainerPort" [plain @91:35..91:48] !<!Ref>
        "Service" [plain @96:3..96:10]: mapping [block]
          "Type" [plain @97:5..97:9]: "AWS::ECS::Service" [plain @97:11..97:28]
          "DependsOn" [plain @98:5..98:14]: "LoadBalancerRule" [plain @98:16..98:32]
          "Properties" [plain @99:5..99:15]: mapping [block]
            "ServiceName" [plain @100:7..100:18]: "ServiceName" [plain @100:25..100:36] !<!Ref>
            "Cluster" [plain @101:7..101:14]: mapping [block] !<!ImportValue>
              "Fn::Join" [plain @102:9..102:17]: sequence [block]
                - ":" [single-quoted @103:13..103:16]
                - sequence [block]
                  - "StackName" [plain @104:20..104:29] !<!Ref>
                  - "ClusterName" [plain @105:15..105:26]
            "LaunchType" [plain @106:7..106:17]: "FARGATE" [plain @106:19..106:26]
            "DeploymentConfiguration" [plain @107:7..107:30]: mapping [block]
              "MaximumPercent" [plain @108:9..108:23]: "200" [plain @108:25..108:28]
              "MinimumHealthyPercent" [plain @109:9..109:30]: "75" [plain @109:32..109:34]
            "DesiredCount" [plain @110:7..110:19]: "DesiredCount" [plain @110:26..110:38] !<!Ref>
            "NetworkConfiguration" [plain @111:7..111:27]: mapping [block]
              "AwsvpcConfiguration" [plain @112:9..112:28]: mapping [block]
                "AssignPublicIp" [plain @113:11..113:25]: "ENABLED" [plain @113:27..113:34]
                "SecurityGroups" [plain @114:11..114:25]: sequence [block]
                  - mapping [block] !<!ImportValue>
                    "Fn::Join" [plain @116:15..116:23]: sequence [block]
                      - ":" [single-quoted @117:19..117:22]
                      - sequence [block]
                        - "StackName" [plain @118:26..118:35] !<!Ref>
                        - "FargateContainerSecurityGroup" [plain @119:21..119:50]
                "Subnets" [plain @120:11..120:18]: sequence [block]
                  - mapping [block] !<!ImportValue>
                    "Fn::Join" [plain @122:15..122:23]: sequence [block]
                      - ":" [single-quoted @123:19..123:22]
                      - sequence [block]
                        - "StackName" [plain @124:26..124:35] !<!Ref>
                        - "PrivateSubnetOne" [plain @125:21..125:37]
                  - mapping [block] !<!ImportValue>
                    "Fn::Join" [plain @127:15..127:23]: sequence [block]
                      - ":" [single-quoted @128:19..128:22]
                      - sequence [block]
                        - "StackName" [plain @129:26..129:35] !<!Ref>
                        - "PrivateSubnetTwo" [plain @130:21..130:37]
            "TaskDefinition" [plain @131:7..131:21]: "TaskDefinition" [plain @131:28..131:42] !<!Ref>
            "LoadBalancers" [plain @132:7..132:20]: sequence [block]
              - mapping [block]
                "ContainerName" [plain @133:11..133:24]: "ServiceName" [plain @133:31..133:42] !<!Ref>
                "ContainerPort" [plain @134:11..134:24]: "ContainerPort" [plain @134:31..134:44] !<!Ref>
                "TargetGroupArn" [plain @135:11..135:25]: "TargetGroup" [plain @135:32..135:43] !<!Ref>
        "TargetGroup" [plain @142:3..142:14]: mapping [block]
          "Type" [plain @143:5..143:9]: "AWS::ElasticLoadBalancingV2::TargetGroup" [plain @143:11..143:51]
          "Properties" [plain @144:5..144:15]: mapping [block]
            "HealthCheckIntervalSeconds" [plain @145:7..145:33]: "6" [plain @145:35..145:36]
            "HealthCheckPath" [plain @146:7..146:22]: "/" [plain @146:24..146:25]
            "HealthCheckProtocol" [plain @147:7..147:26]: "HTTP" [plain @147:28..147:32]
            "HealthCheckTimeoutSeconds" [plain @148:7..148:32]: "5" [plain @148:34..148:35]
            "HealthyThresholdCount" [plain @149:7..149:28]: "2" [plain @149:30..149:31]
            "TargetType" [plain @150:7..150:17]: "ip" [plain @150:19..150:21]
            "Name" [plain @151:7..151:11]: "ServiceName" [plain @151:18..151:29] !<!Ref>
            "Port" [plain @152:7..152:11]: "ContainerPort" [plain @152:18..152:31] !<!Ref>
            "Protocol" [plain @153:7..153:15]: "HTTP" [plain @153:17..153:21]
            "UnhealthyThresholdCount" [plain @154:7..154:30]: "2" [plain @154:32..154:33]
            "VpcId" [plain @155:7..155:12]: mapping [block] !<!ImportValue>
              "Fn::Join" [plain @156:9..156:17]: sequence [block]
                - ":" [single-quoted @157:13..157:16]
                - sequence [block]
                  - "StackName" [plain @158:20..158:29] !<!Ref>
                  - "VPCId" [plain @159:15..159:20]
        "LoadBalancerRule" [plain @162:3..162:19]: mapping [block]
          "Type" [plain @163:5..163:9]: "AWS::ElasticLoadBalancingV2::ListenerRule" [plain @163:11..163:52]
          "Properties" [plain @164:5..164:15]: mapping [block]
            "Actions" [plain @165:7..165:14]: sequence [block]
              - mapping [block]
                "TargetGroupArn" [plain @166:11..166:25]: "TargetGroup" [plain @166:32..166:43] !<!Ref>
                "Type" [plain @167:11..167:15]: "forward" [plain @167:17..167:24]
            "Conditions" [plain @168:7..168:17]: sequence [block]
              - mapping [block]
                "Field" [plain @169:11..169:16]: "path-pattern" [plain @169:18..169:30]
                "Values" [plain @170:11..170:17]: sequence [block]
                  - "Path" [plain @171:20..171:24] !<!Ref>
            "ListenerArn" [plain @172:7..172:18]: mapping [block] !<!ImportValue>
              "Fn::Join" [plain @173:9..173:17]: sequence [block]
                - ":" [single-quoted @174:13..174:16]
                - sequence [block]
                  - "StackName" [plain @175:20..175:29] !<!Ref>
                  - "PublicListener" [plain @176:15..176:29]
            "Priority" [plain @177:7..177:15]: "Priority" [plain @177:22..177:30] !<!Ref>
    "#);
}
