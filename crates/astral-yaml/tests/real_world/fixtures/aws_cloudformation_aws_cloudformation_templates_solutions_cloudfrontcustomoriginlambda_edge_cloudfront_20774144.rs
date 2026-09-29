#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-cloudfrontcustomoriginlambda-edge-cloudfront-20774144"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "CI/CD optimized AWS CloudFormation Sample Template for AWS CloudFront Distribution with Custom Origin with an example of using the AWS Application Load Balancer (ALB) and a basic Amazon EC2 Instance. AWS CloudFront Distribution is associated with Lambda@Edge for Security Headers inspection. In addition, AWS CloudFormation Template will provision an Examples of necessary IAM, S3, KMS and Security Groups resources. ### Before deployment please make sure that all parameters are reviewed and updated according the specific use case. ### **WARNING** This template creates one Amazon EC2 instance and an Application Load Balancer, KMS Keys, S3 bucket, CloudFront Distribution resources. You will be billed for the AWS resources used if you create a stack from this template." [single-quoted @3:14..3:789]
      "Metadata" [plain @5:1..5:9]: mapping [block]
        "cfn-lint" [plain @6:3..6:11]: mapping [block]
          "config" [plain @7:5..7:11]: mapping [block]
            "regions" [plain @8:7..8:14]: sequence [block]
              - "us-east-1" [plain @9:11..9:20]
              - "us-west-2" [plain @10:11..10:20]
      "Parameters" [plain @12:1..12:11]: mapping [block]
        "Environment" [plain @13:3..13:14]: mapping [block]
          "Description" [plain @14:5..14:16]: "Please specify the target environment." [plain @14:18..14:56]
          "Type" [plain @15:5..15:9]: "String" [plain @15:11..15:17]
          "AllowedValues" [plain @16:5..16:18]: sequence [block]
            - "prod" [plain @17:9..17:13]
            - "staging" [plain @18:9..18:16]
            - "dev" [plain @19:9..19:12]
            - "qa" [plain @20:9..20:11]
          "Default" [plain @21:5..21:12]: "dev" [plain @21:14..21:17]
        "VpcId" [plain @23:3..23:8]: mapping [block]
          "Description" [plain @24:5..24:16]: "Please specify the VPC ID." [plain @24:18..24:44]
          "Type" [plain @25:5..25:9]: "AWS::EC2::VPC::Id" [plain @25:11..25:28]
          "ConstraintDescription" [plain @26:5..26:26]: "Must be a valid VPC ID" [double-quoted @26:28..26:52]
        "PublicSubnetId1" [plain @28:3..28:18]: mapping [block]
          "Description" [plain @29:5..29:16]: "Please specify first public subnet ID." [plain @29:18..29:56]
          "Type" [plain @30:5..30:9]: "AWS::EC2::Subnet::Id" [plain @30:11..30:31]
          "ConstraintDescription" [plain @31:5..31:26]: "Must be a valid subnet ID in the selected VPC" [double-quoted @31:28..31:75]
        "PublicSubnetId2" [plain @33:3..33:18]: mapping [block]
          "Description" [plain @34:5..34:16]: "Please specify second public subnet ID." [plain @34:18..34:57]
          "Type" [plain @35:5..35:9]: "AWS::EC2::Subnet::Id" [plain @35:11..35:31]
          "ConstraintDescription" [plain @36:5..36:26]: "Must be a valid subnet ID in the selected VPC" [double-quoted @36:28..36:75]
        "AppName" [plain @39:3..39:10]: mapping [block]
          "Description" [plain @40:5..40:16]: "Application environment name." [plain @40:18..40:47]
          "Type" [plain @41:5..41:9]: "String" [plain @41:11..41:17]
          "Default" [plain @42:5..42:12]: "example" [plain @42:14..42:21]
        "AlternateDomainNames" [plain @44:3..44:23]: mapping [block]
          "Description" [plain @45:5..45:16]: "CNAMEs (alternate domain names), if any, for the distribution. Example. mydomain.com" [plain @45:18..45:102]
          "Type" [plain @46:5..46:9]: "String" [plain @46:11..46:17]
          "Default" [plain @47:5..47:12]: "name.domain.com" [plain @47:14..47:29]
        "ACMCertificateIdentifier" [plain @49:3..49:27]: mapping [block]
          "Description" [plain @50:5..50:16]: "The AWS Certificate Manager (ACM) certificate identifier." [plain @50:18..50:75]
          "Type" [plain @51:5..51:9]: "String" [plain @51:11..51:17]
          "Default" [plain @52:5..52:12]: "1234567890abcdefgh" [plain @52:14..52:32]
        "LambdaEventType" [plain @54:3..54:18]: mapping [block]
          "Description" [plain @55:5..55:16]: "Please specify the event type that triggers a Lambda function invocation." [plain @55:18..55:91]
          "Type" [plain @56:5..56:9]: "String" [plain @56:11..56:17]
          "AllowedValues" [plain @57:5..57:18]: sequence [block]
            - "viewer-request" [plain @58:9..58:23]
            - "origin-request" [plain @59:9..59:23]
            - "origin-response" [plain @60:9..60:24]
            - "viewer-response" [plain @61:9..61:24]
          "Default" [plain @62:5..62:12]: "viewer-response" [plain @62:14..62:29]
        "IPV6Enabled" [plain @64:3..64:14]: mapping [block]
          "Description" [plain @65:5..65:16]: "Should CloudFront to respond to IPv6 DNS requests with an IPv6 address for your distribution." [plain @65:18..65:111]
          "Type" [plain @66:5..66:9]: "String" [plain @66:11..66:17]
          "AllowedValues" [plain @67:5..67:18]: sequence [block]
            - "true" [double-quoted @68:9..68:15]
            - "false" [double-quoted @69:9..69:16]
          "Default" [plain @70:5..70:12]: "true" [double-quoted @70:14..70:20]
        "EC2ImageId" [plain @72:3..72:13]: mapping [block]
          "Description" [plain @73:5..73:16]: "EC2 AMI Id" [plain @73:18..73:28]
          "Type" [plain @74:5..74:9]: "AWS::EC2::Image::Id" [plain @74:11..74:30]
          "Default" [plain @75:5..75:12]: "ami-0d85a662720db9789" [plain @75:14..75:35]
        "EC2InstanceType" [plain @77:3..77:18]: mapping [block]
          "Description" [plain @78:5..78:16]: "Amazon EC2 instance type." [plain @78:18..78:43]
          "Type" [plain @79:5..79:9]: "String" [plain @79:11..79:17]
          "AllowedValues" [plain @80:5..80:18]: sequence [block]
            - "t2.small" [plain @81:9..81:17]
            - "t2.medium" [plain @82:9..82:18]
            - "t2.large" [plain @83:9..83:17]
            - "t2.xlarge" [plain @84:9..84:18]
            - "t2.2xlarge" [plain @85:9..85:19]
            - "m4.large" [plain @86:9..86:17]
            - "m4.xlarge" [plain @87:9..87:18]
            - "m4.2xlarge" [plain @88:9..88:19]
            - "m4.4xlarge" [plain @89:9..89:19]
            - "m4.10xlarge" [plain @90:9..90:20]
            - "m4.16xlarge" [plain @91:9..91:20]
            - "m5.large" [plain @92:9..92:17]
            - "m5.xlarge" [plain @93:9..93:18]
            - "m5.2xlarge" [plain @94:9..94:19]
            - "m5.4xlarge" [plain @95:9..95:19]
            - "m5.12xlarge" [plain @96:9..96:20]
            - "m5.24xlarge" [plain @97:9..97:20]
            - "m5d.large" [plain @98:9..98:18]
            - "m5d.xlarge" [plain @99:9..99:19]
            - "m5d.2xlarge" [plain @100:9..100:20]
            - "m5d.4xlarge" [plain @101:9..101:20]
            - "m5d.12xlarge" [plain @102:9..102:21]
            - "m5d.24xlarge" [plain @103:9..103:21]
          "Default" [plain @104:5..104:12]: "t2.small" [plain @104:14..104:22]
        "KeyPairName" [plain @106:3..106:14]: mapping [block]
          "Description" [plain @107:5..107:16]: "EC2 KeyPair." [plain @107:18..107:30]
          "Type" [plain @108:5..108:9]: "AWS::EC2::KeyPair::KeyName" [plain @108:11..108:37]
          "ConstraintDescription" [plain @109:5..109:26]: "Must be the name of an existing EC2 KeyPair" [double-quoted @109:28..109:73]
        "BootVolSize" [plain @111:3..111:14]: mapping [block]
          "Description" [plain @112:5..112:16]: "EC2 Instance Boot volume size." [plain @112:18..112:48]
          "Type" [plain @113:5..113:9]: "String" [plain @113:11..113:17]
          "Default" [plain @114:5..114:12]: "100" [double-quoted @114:14..114:19]
        "BootVolType" [plain @116:3..116:14]: mapping [block]
          "Description" [plain @117:5..117:16]: "EC2 Instance Boot volume type." [plain @117:18..117:48]
          "Type" [plain @118:5..118:9]: "String" [plain @118:11..118:17]
          "AllowedValues" [plain @119:5..119:18]: sequence [block]
            - "gp2" [plain @120:9..120:12]
            - "io1" [plain @121:9..121:12]
            - "sc1" [plain @122:9..122:12]
            - "st1" [plain @123:9..123:12]
          "Default" [plain @124:5..124:12]: "gp2" [plain @124:14..124:17]
        "ALBType" [plain @126:3..126:10]: mapping [block]
          "Description" [plain @127:5..127:16]: "AWS Load Balancer Type." [plain @127:18..127:41]
          "Type" [plain @128:5..128:9]: "String" [plain @128:11..128:17]
          "AllowedValues" [plain @129:5..129:18]: sequence [block]
            - "application" [plain @130:9..130:20]
            - "network" [plain @131:9..131:16]
          "Default" [plain @132:5..132:12]: "application" [plain @132:14..132:25]
        "OriginALBTGPort" [plain @134:3..134:18]: mapping [block]
          "Description" [plain @135:5..135:16]: "Port number the application is running on, for Origin ALB Target Group and Health Check port." [plain @135:18..135:111]
          "Type" [plain @136:5..136:9]: "String" [plain @136:11..136:17]
          "Default" [plain @137:5..137:12]: "8080" [double-quoted @137:14..137:20]
        "OriginProtocolPolicy" [plain @139:3..139:23]: mapping [block]
          "Description" [plain @140:5..140:16]: "CloudFront Origin Protocol Policy to apply to your origin." [plain @140:18..140:76]
          "Type" [plain @141:5..141:9]: "String" [plain @141:11..141:17]
          "AllowedValues" [plain @142:5..142:18]: sequence [block]
            - "http-only" [plain @143:9..143:18]
            - "match-viewer" [plain @144:9..144:21]
            - "https-only" [plain @145:9..145:19]
          "Default" [plain @146:5..146:12]: "http-only" [plain @146:14..146:23]
        "Compress" [plain @148:3..148:11]: mapping [block]
          "Description" [plain @149:5..149:16]: "CloudFront should support gzip compression requests: Accept-Encoding: gzip." [single-quoted @149:18..149:95]
          "Type" [plain @150:5..150:9]: "String" [plain @150:11..150:17]
          "AllowedValues" [plain @151:5..151:18]: sequence [block]
            - "true" [double-quoted @152:9..152:15]
            - "false" [double-quoted @153:9..153:16]
          "Default" [plain @154:5..154:12]: "false" [double-quoted @154:14..154:21]
        "DefaultTTL" [plain @156:3..156:13]: mapping [block]
          "Description" [plain @157:5..157:16]: "The default time in seconds that objects stay in CloudFront caches before CloudFront forwards another request to your custom origin. By default, AWS CloudFormation specifies 86400 seconds (one day)." [plain @157:18..157:216]
          "Type" [plain @158:5..158:9]: "String" [plain @158:11..158:17]
          "Default" [plain @159:5..159:12]: "0" [double-quoted @159:14..159:17]
        "MaxTTL" [plain @161:3..161:9]: mapping [block]
          "Description" [plain @162:5..162:16]: "The maximum time in seconds that objects stay in CloudFront caches before CloudFront forwards another request to your custom origin. By default, AWS CloudFormation specifies 31536000 seconds (one year)." [plain @162:18..162:220]
          "Type" [plain @163:5..163:9]: "String" [plain @163:11..163:17]
          "Default" [plain @164:5..164:12]: "0" [double-quoted @164:14..164:17]
        "MinTTL" [plain @166:3..166:9]: mapping [block]
          "Description" [plain @167:5..167:16]: "The minimum amount of time that you want objects to stay in the cache before CloudFront queries your origin to see whether the object has been updated." [plain @167:18..167:169]
          "Type" [plain @168:5..168:9]: "String" [plain @168:11..168:17]
          "Default" [plain @169:5..169:12]: "0" [double-quoted @169:14..169:17]
        "QueryString" [plain @171:3..171:14]: mapping [block]
          "Description" [plain @172:5..172:16]: "CIndicates whether you want CloudFront to forward query strings to the origin that is associated with this cache behavior." [plain @172:18..172:140]
          "Type" [plain @173:5..173:9]: "String" [plain @173:11..173:17]
          "AllowedValues" [plain @174:5..174:18]: sequence [block]
            - "true" [double-quoted @175:9..175:15]
            - "false" [double-quoted @176:9..176:16]
          "Default" [plain @177:5..177:12]: "true" [double-quoted @177:14..177:20]
        "ForwardCookies" [plain @179:3..179:17]: mapping [block]
          "Description" [plain @180:5..180:16]: "Forwards specified cookies to the origin of the cache behavior." [plain @180:18..180:81]
          "Type" [plain @181:5..181:9]: "String" [plain @181:11..181:17]
          "AllowedValues" [plain @182:5..182:18]: sequence [block]
            - "all" [plain @183:9..183:12]
            - "whitelist" [plain @184:9..184:18]
            - "none" [plain @185:9..185:13]
          "Default" [plain @186:5..186:12]: "all" [plain @186:14..186:17]
        "ViewerProtocolPolicy" [plain @188:3..188:23]: mapping [block]
          "Description" [plain @189:5..189:16]: "The protocol that users can use to access the files in the origin that you specified in the TargetOriginId property when the default cache behavior is applied to a request." [plain @189:18..189:190]
          "Type" [plain @190:5..190:9]: "String" [plain @190:11..190:17]
          "AllowedValues" [plain @191:5..191:18]: sequence [block]
            - "redirect-to-https" [plain @192:9..192:26]
            - "allow-all" [plain @193:9..193:18]
            - "https-only" [plain @194:9..194:19]
          "Default" [plain @195:5..195:12]: "redirect-to-https" [plain @195:14..195:31]
        "PriceClass" [plain @197:3..197:13]: mapping [block]
          "Description" [plain @198:5..198:16]: "The price class that corresponds with the maximum price that you want to pay for CloudFront service. If you specify PriceClass_All, CloudFront responds to requests for your objects from all CloudFront edge locations." [plain @198:18..198:234]
          "Type" [plain @199:5..199:9]: "String" [plain @199:11..199:17]
          "AllowedValues" [plain @200:5..200:18]: sequence [block]
            - "PriceClass_All" [plain @201:9..201:23]
            - "PriceClass_100" [plain @202:9..202:23]
            - "PriceClass_200" [plain @203:9..203:23]
          "Default" [plain @204:5..204:12]: "PriceClass_All" [plain @204:14..204:28]
        "SslSupportMethod" [plain @206:3..206:19]: mapping [block]
          "Description" [plain @207:5..207:16]: "Specifies how CloudFront serves HTTPS requests." [plain @207:18..207:65]
          "Type" [plain @208:5..208:9]: "String" [plain @208:11..208:17]
          "AllowedValues" [plain @209:5..209:18]: sequence [block]
            - "sni-only" [plain @210:9..210:17]
            - "vip" [plain @211:9..211:12]
          "Default" [plain @212:5..212:12]: "sni-only" [plain @212:14..212:22]
        "MinimumProtocolVersion" [plain @214:3..214:25]: mapping [block]
          "Description" [plain @215:5..215:16]: "The minimum version of the SSL protocol that you want CloudFront to use for HTTPS connections." [plain @215:18..215:112]
          "Type" [plain @216:5..216:9]: "String" [plain @216:11..216:17]
          "AllowedValues" [plain @217:5..217:18]: sequence [block]
            - "TLSv1" [plain @218:9..218:14]
            - "TLSv1.2_2018" [plain @219:9..219:21]
            - "TLSv1.1_2016" [plain @220:9..220:21]
            - "TLSv1_2016" [plain @221:9..221:19]
            - "SSLv3" [plain @222:9..222:14]
          "Default" [plain @223:5..223:12]: "TLSv1" [plain @223:14..223:19]
        "OriginKeepaliveTimeout" [plain @225:3..225:25]: mapping [block]
          "Description" [plain @226:5..226:16]: "You can create a custom keep-alive timeout. All timeout units are in seconds. The default keep-alive timeout is 5 seconds, but you can configure custom timeout lengths. The minimum timeout length is 1 second; the maximum is 60 seconds." [plain @226:18..226:253]
          "Type" [plain @227:5..227:9]: "String" [plain @227:11..227:17]
          "Default" [plain @228:5..228:12]: "60" [double-quoted @228:14..228:18]
        "OriginReadTimeout" [plain @230:3..230:20]: mapping [block]
          "Description" [plain @231:5..231:16]: "You can create a custom origin read timeout. All timeout units are in seconds. The default origin read timeout is 30 seconds, but you can configure custom timeout lengths. The minimum timeout length is 4 seconds; the maximum is 60 seconds." [plain @231:18..231:257]
          "Type" [plain @232:5..232:9]: "String" [plain @232:11..232:17]
          "Default" [plain @233:5..233:12]: "30" [double-quoted @233:14..233:18]
        "ALBScheme" [plain @235:3..235:12]: mapping [block]
          "Description" [plain @236:5..236:16]: "Origin ALB scheme." [plain @236:18..236:36]
          "Type" [plain @237:5..237:9]: "String" [plain @237:11..237:17]
          "AllowedValues" [plain @238:5..238:18]: sequence [block]
            - "internet-facing" [plain @239:9..239:24]
            - "internal" [plain @240:9..240:17]
          "Default" [plain @241:5..241:12]: "internet-facing" [plain @241:14..241:29]
        "ALBTargetGroupHealthCheckIntervalSeconds" [plain @243:3..243:43]: mapping [block]
          "Description" [plain @244:5..244:16]: "Origin ALB Target Group Health Check Interval in Seconds." [plain @244:18..244:75]
          "Type" [plain @245:5..245:9]: "String" [plain @245:11..245:17]
          "Default" [plain @246:5..246:12]: "30" [double-quoted @246:14..246:18]
        "ALBTargetGroupHealthCheckTimeoutSeconds" [plain @248:3..248:42]: mapping [block]
          "Description" [plain @249:5..249:16]: "Origin ALB Target Group Health Check Timeout in Seconds." [plain @249:18..249:74]
          "Type" [plain @250:5..250:9]: "String" [plain @250:11..250:17]
          "Default" [plain @251:5..251:12]: "5" [double-quoted @251:14..251:17]
        "ALBTargetGroupHealthyThresholdCount" [plain @253:3..253:38]: mapping [block]
          "Description" [plain @254:5..254:16]: "Origin ALB Target Group Healthy Threshold Count." [plain @254:18..254:66]
          "Type" [plain @255:5..255:9]: "String" [plain @255:11..255:17]
          "Default" [plain @256:5..256:12]: "5" [double-quoted @256:14..256:17]
        "ALBTargetGroupUnhealthyThresholdCount" [plain @258:3..258:40]: mapping [block]
          "Description" [plain @259:5..259:16]: "Origin ALB Target Group Unhealthy Threshold Count." [plain @259:18..259:68]
          "Type" [plain @260:5..260:9]: "String" [plain @260:11..260:17]
          "Default" [plain @261:5..261:12]: "2" [double-quoted @261:14..261:17]
        "ALBAttributeIdleTimeOut" [plain @263:3..263:26]: mapping [block]
          "Description" [plain @264:5..264:16]: "Origin ALB Target Group Unhealthy Threshold Count." [plain @264:18..264:68]
          "Type" [plain @265:5..265:9]: "String" [plain @265:11..265:17]
          "Default" [plain @266:5..266:12]: "60" [double-quoted @266:14..266:18]
        "ALBAttributeDeletionProtection" [plain @268:3..268:33]: mapping [block]
          "Description" [plain @269:5..269:16]: "Origin ALB Target Group Unhealthy Threshold Count." [plain @269:18..269:68]
          "Type" [plain @270:5..270:9]: "String" [plain @270:11..270:17]
          "AllowedValues" [plain @271:5..271:18]: sequence [block]
            - "true" [double-quoted @272:9..272:15]
            - "false" [double-quoted @273:9..273:16]
          "Default" [plain @274:5..274:12]: "false" [double-quoted @274:14..274:21]
        "ALBAttributeRoutingHttp2" [plain @276:3..276:27]: mapping [block]
          "Description" [plain @277:5..277:16]: "Origin ALB Target Group Unhealthy Threshold Count." [plain @277:18..277:68]
          "Type" [plain @278:5..278:9]: "String" [plain @278:11..278:17]
          "AllowedValues" [plain @279:5..279:18]: sequence [block]
            - "true" [double-quoted @280:9..280:15]
            - "false" [double-quoted @281:9..281:16]
          "Default" [plain @282:5..282:12]: "true" [double-quoted @282:14..282:20]
        "ALBTargetGroupAttributeDeregistration" [plain @284:3..284:40]: mapping [block]
          "Description" [plain @285:5..285:16]: "Origin ALB Target Group Deregistration Timeout." [plain @285:18..285:65]
          "Type" [plain @286:5..286:9]: "String" [plain @286:11..286:17]
          "Default" [plain @287:5..287:12]: "300" [double-quoted @287:14..287:19]
        "HealthCheckProtocol" [plain @289:3..289:22]: mapping [block]
          "Description" [plain @290:5..290:16]: "Origin ALB Target Group Health Check Protocol." [plain @290:18..290:64]
          "Type" [plain @291:5..291:9]: "String" [plain @291:11..291:17]
          "AllowedValues" [plain @292:5..292:18]: sequence [block]
            - "HTTPS" [plain @293:9..293:14]
            - "HTTP" [plain @294:9..294:13]
          "Default" [plain @295:5..295:12]: "HTTP" [plain @295:14..295:18]
        "HealthCheckPath" [plain @297:3..297:18]: mapping [block]
          "Description" [plain @298:5..298:16]: "Origin ALB Target Group Health Check Path." [plain @298:18..298:60]
          "Type" [plain @299:5..299:9]: "String" [plain @299:11..299:17]
          "Default" [plain @300:5..300:12]: "/health.html" [plain @300:14..300:26]
        "LoggingBucketVersioning" [plain @302:3..302:26]: mapping [block]
          "Description" [plain @303:5..303:16]: "The versioning state of an Amazon S3 bucket. If you enable versioning, you must suspend versioning to disable it." [plain @303:18..303:131]
          "Type" [plain @304:5..304:9]: "String" [plain @304:11..304:17]
          "AllowedValues" [plain @305:5..305:18]: sequence [block]
            - "Enabled" [plain @306:9..306:16]
            - "Suspended" [plain @307:9..307:18]
          "Default" [plain @308:5..308:12]: "Suspended" [plain @308:14..308:23]
      "Resources" [plain @310:1..310:10]: mapping [block]
        "AdministratorAccessIAMRole" [plain @313:3..313:29]: mapping [block]
          "Type" [plain @314:5..314:9]: "AWS::IAM::Role" [plain @314:11..314:25]
          "Properties" [plain @315:5..315:15]: mapping [block]
            "RoleName" [plain @316:7..316:15]: "AdministratorAccess-${AppName}" [plain @316:22..316:52] !<!Sub>
            "ManagedPolicyArns" [plain @317:7..317:24]: sequence [block]
              - "arn:${AWS::Partition}:iam::aws:policy/AdministratorAccess" [plain @318:16..318:73] !<!Sub>
            "AssumeRolePolicyDocument" [plain @319:7..319:31]: mapping [block]
              "Version" [plain @320:9..320:16]: "2012-10-17" [double-quoted @320:18..320:30]
              "Statement" [plain @321:9..321:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @322:13..322:19]: "Allow" [plain @322:21..322:26]
                  "Principal" [plain @323:13..323:22]: mapping [block]
                    "Service" [plain @324:15..324:22]: sequence [block]
                      - "ec2.amazonaws.com" [plain @325:19..325:36]
                  "Action" [plain @326:13..326:19]: sequence [block]
                    - "sts:AssumeRole" [plain @327:17..327:31]
            "Path" [plain @328:7..328:11]: "/" [plain @328:13..328:14]
        "LambdaEdgeIAMRole" [plain @331:3..331:20]: mapping [block]
          "Type" [plain @332:5..332:9]: "AWS::IAM::Role" [plain @332:11..332:25]
          "Properties" [plain @333:5..333:15]: mapping [block]
            "RoleName" [plain @334:7..334:15]: "${AppName}-iam-lambda-edge-role-${Environment}" [plain @334:22..334:68] !<!Sub>
            "AssumeRolePolicyDocument" [plain @335:7..335:31]: mapping [block]
              "Version" [plain @336:9..336:16]: "2012-10-17" [double-quoted @336:18..336:30]
              "Statement" [plain @337:9..337:18]: sequence [block]
                - mapping [block]
                  "Sid" [plain @338:13..338:16]: "AllowLambdaServiceToAssumeRole" [plain @338:18..338:48]
                  "Effect" [plain @339:13..339:19]: "Allow" [plain @339:21..339:26]
                  "Principal" [plain @340:13..340:22]: mapping [block]
                    "Service" [plain @341:15..341:22]: sequence [block]
                      - "edgelambda.amazonaws.com" [plain @342:19..342:43]
                      - "lambda.amazonaws.com" [plain @343:19..343:39]
                  "Action" [plain @344:13..344:19]: sequence [block]
                    - "sts:AssumeRole" [plain @345:17..345:31]
            "ManagedPolicyArns" [plain @346:7..346:24]: sequence [block]
              - "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole" [plain @347:11..347:75]
              - "arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess" [plain @348:11..348:57]
            "Path" [plain @349:7..349:11]: "/" [plain @349:13..349:14]
            "Policies" [plain @350:7..350:15]: sequence [block]
              - mapping [block]
                "PolicyName" [plain @351:11..351:21]: "PublishNewLambdaEdgeVersion" [plain @351:23..351:50]
                "PolicyDocument" [plain @352:11..352:25]: mapping [block]
                  "Version" [plain @353:13..353:20]: "2012-10-17" [double-quoted @353:22..353:34]
                  "Statement" [plain @354:13..354:22]: sequence [block]
                    - mapping [block]
                      "Effect" [plain @355:17..355:23]: "Allow" [plain @355:25..355:30]
                      "Action" [plain @356:17..356:23]: sequence [block]
                        - "lambda:PublishVersion" [plain @357:21..357:42]
                      "Resource" [plain @358:17..358:25]: "*" [single-quoted @358:27..358:30]
        "LoggingBucketKMSKey" [plain @361:3..361:22]: mapping [block]
          "Type" [plain @362:5..362:9]: "AWS::KMS::Key" [plain @362:11..362:24]
          "DependsOn" [plain @363:5..363:14]: "AdministratorAccessIAMRole" [plain @363:16..363:42]
          "Properties" [plain @364:5..364:15]: mapping [block]
            "Description" [plain @365:7..365:18]: "Logging S3 Bucket KMS Key" [plain @365:20..365:45]
            "Enabled" [plain @366:7..366:14]: "true" [plain @366:16..366:20]
            "EnableKeyRotation" [plain @367:7..367:24]: "true" [plain @367:26..367:30]
            "KeyPolicy" [plain @368:7..368:16]: mapping [block]
              "Version" [plain @369:9..369:16]: "2012-10-17" [double-quoted @369:18..369:30]
              "Statement" [plain @370:9..370:18]: sequence [block]
                - mapping [block]
                  "Sid" [plain @371:13..371:16]: "Enable IAM policies to allow access to the Key" [plain @371:18..371:64]
                  "Effect" [plain @372:13..372:19]: "Allow" [plain @372:21..372:26]
                  "Principal" [plain @373:13..373:22]: mapping [block]
                    "AWS" [plain @374:15..374:18]: "arn:${AWS::Partition}:iam::${AWS::AccountId}:root" [plain @374:25..374:74] !<!Sub>
                  "Action" [plain @375:13..375:19]: sequence [block]
                    - "kms:*" [plain @376:17..376:22]
                  "Resource" [plain @377:13..377:21]: "*" [single-quoted @377:23..377:26]
                - mapping [block]
                  "Sid" [plain @378:13..378:16]: "Allow administration of the key" [plain @378:18..378:49]
                  "Effect" [plain @379:13..379:19]: "Allow" [plain @379:21..379:26]
                  "Principal" [plain @380:13..380:22]: mapping [block]
                    "AWS" [plain @381:15..381:18]: sequence [block]
                      - "arn:${AWS::Partition}:iam::${AWS::AccountId}:role/AdministratorAccess-${AppName}" [plain @382:24..382:104] !<!Sub>
                  "Action" [plain @383:13..383:19]: sequence [block]
                    - "kms:Put*" [plain @384:17..384:25]
                    - "kms:ScheduleKeyDeletion" [plain @385:17..385:40]
                    - "kms:CancelKeyDeletion" [plain @386:17..386:38]
                    - "kms:Describe*" [plain @387:17..387:30]
                    - "kms:Revoke*" [plain @388:17..388:28]
                    - "kms:Disable*" [plain @389:17..389:29]
                    - "kms:Enable*" [plain @390:17..390:28]
                    - "kms:Delete*" [plain @391:17..391:28]
                    - "kms:List*" [plain @392:17..392:26]
                    - "kms:Update*" [plain @393:17..393:28]
                    - "kms:Create*" [plain @394:17..394:28]
                  "Resource" [plain @395:13..395:21]: "*" [single-quoted @395:23..395:26]
        "LoggingBucketKMSKeyAlias" [plain @398:3..398:27]: mapping [block]
          "Type" [plain @399:5..399:9]: "AWS::KMS::Alias" [plain @399:11..399:26]
          "Properties" [plain @400:5..400:15]: mapping [block]
            "AliasName" [plain @401:7..401:16]: "alias/${AppName}/${Environment}/s3-logging-kms" [plain @401:23..401:69] !<!Sub>
            "TargetKeyId" [plain @402:7..402:18]: "${LoggingBucketKMSKey}" [plain @402:25..402:47] !<!Sub>
        "LoggingBucket" [plain @405:3..405:16]: mapping [block]
          "DeletionPolicy" [plain @406:5..406:19]: "Retain" [plain @406:21..406:27]
          "UpdateReplacePolicy" [plain @407:5..407:24]: "Retain" [plain @407:26..407:32]
          "Type" [plain @408:5..408:9]: "AWS::S3::Bucket" [plain @408:11..408:26]
          "DependsOn" [plain @409:5..409:14]: "LoggingBucketKMSKey" [plain @409:16..409:35]
          "Metadata" [plain @410:5..410:13]: mapping [block]
            "guard" [plain @411:7..411:12]: mapping [block]
              "SuppressedRules" [plain @412:9..412:24]: sequence [block]
                - "S3_BUCKET_DEFAULT_LOCK_ENABLED" [plain @413:13..413:43]
                - "S3_BUCKET_VERSIONING_ENABLED" [plain @414:13..414:41]
                - "S3_BUCKET_REPLICATION_ENABLED" [plain @415:13..415:42]
                - "S3_BUCKET_LOGGING_ENABLED" [plain @416:13..416:38]
          "Properties" [plain @417:5..417:15]: mapping [block]
            "BucketName" [plain @418:7..418:17]: "${AppName}-logging-${Environment}-${AWS::AccountId}-${AWS::Region}" [plain @418:24..418:90] !<!Sub>
            "OwnershipControls" [plain @419:7..419:24]: mapping [block]
              "Rules" [plain @420:9..420:14]: sequence [block]
                - mapping [block]
                  "ObjectOwnership" [plain @421:13..421:28]: "ObjectWriter" [plain @421:30..421:42]
            "PublicAccessBlockConfiguration" [plain @422:7..422:37]: mapping [block]
              "BlockPublicAcls" [plain @423:9..423:24]: "true" [plain @423:26..423:30]
              "BlockPublicPolicy" [plain @424:9..424:26]: "true" [plain @424:28..424:32]
              "IgnorePublicAcls" [plain @425:9..425:25]: "true" [plain @425:27..425:31]
              "RestrictPublicBuckets" [plain @426:9..426:30]: "true" [plain @426:32..426:36]
            "AccessControl" [plain @427:7..427:20]: "LogDeliveryWrite" [plain @427:22..427:38]
            "BucketEncryption" [plain @428:7..428:23]: mapping [block]
              "ServerSideEncryptionConfiguration" [plain @429:9..429:42]: sequence [block]
                - mapping [block]
                  "ServerSideEncryptionByDefault" [plain @430:13..430:42]: mapping [block]
                    "KMSMasterKeyID" [plain @431:15..431:29]: "LoggingBucketKMSKey.Arn" [plain @431:39..431:62] !<!GetAtt>
                    "SSEAlgorithm" [plain @432:15..432:27]: "aws:kms" [plain @432:29..432:36]
            "VersioningConfiguration" [plain @433:7..433:30]: mapping [block]
              "Status" [plain @434:9..434:15]: "LoggingBucketVersioning" [plain @434:22..434:45] !<!Ref>
        "LoggingBucketPolicy" [plain @437:3..437:22]: mapping [block]
          "Type" [plain @438:5..438:9]: "AWS::S3::BucketPolicy" [plain @438:11..438:32]
          "Properties" [plain @439:5..439:15]: mapping [block]
            "Bucket" [plain @440:7..440:13]: "LoggingBucket" [plain @440:20..440:33] !<!Ref>
            "PolicyDocument" [plain @441:7..441:21]: mapping [block]
              "Version" [plain @442:9..442:16]: "2012-10-17" [double-quoted @442:18..442:30]
              "Statement" [plain @443:9..443:18]: sequence [block]
                - mapping [block]
                  "Sid" [plain @444:13..444:16]: "LoggingBucketPermissions" [plain @444:18..444:42]
                  "Effect" [plain @445:13..445:19]: "Allow" [plain @445:21..445:26]
                  "Principal" [plain @446:13..446:22]: mapping [block]
                    "AWS" [plain @447:15..447:18]: "arn:${AWS::Partition}:iam::${AWS::AccountId}:root" [plain @447:25..447:74] !<!Sub>
                  "Action" [plain @448:13..448:19]: "s3:PutObject" [plain @448:21..448:33]
                  "Resource" [plain @449:13..449:21]: sequence [block]
                    - "arn:${AWS::Partition}:s3:::${LoggingBucket}/AWSLogs/${AWS::AccountId}/*" [plain @450:22..450:93] !<!Sub>
                - mapping [block]
                  "Action" [plain @451:13..451:19]: "s3:*" [plain @451:21..451:25]
                  "Condition" [plain @452:13..452:22]: mapping [block]
                    "Bool" [plain @453:15..453:19]: mapping [block]
                      "aws:SecureTransport" [plain @454:17..454:36]: "false" [plain @454:38..454:43]
                  "Effect" [plain @455:13..455:19]: "Deny" [plain @455:21..455:25]
                  "Principal" [plain @456:13..456:22]: mapping [block]
                    "AWS" [plain @457:15..457:18]: "*" [single-quoted @457:20..457:23]
                  "Resource" [plain @458:13..458:21]: sequence [block]
                    - "arn:${AWS::Partition}:s3:::${LoggingBucket}/AWSLogs/${AWS::AccountId}/*" [plain @459:22..459:93] !<!Sub>
        "EC2Instance" [plain @462:3..462:14]: mapping [block]
          "Type" [plain @463:5..463:9]: "AWS::EC2::Instance" [plain @463:11..463:29]
          "Properties" [plain @464:5..464:15]: mapping [block]
            "ImageId" [plain @465:7..465:14]: "EC2ImageId" [plain @465:21..465:31] !<!Ref>
            "InstanceType" [plain @466:7..466:19]: "EC2InstanceType" [plain @466:26..466:41] !<!Ref>
            "SubnetId" [plain @467:7..467:15]: "PublicSubnetId1" [plain @467:22..467:37] !<!Ref>
            "BlockDeviceMappings" [plain @468:7..468:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @469:11..469:21]: "/dev/sda1" [plain @469:23..469:32]
                "Ebs" [plain @470:11..470:14]: mapping [block]
                  "VolumeSize" [plain @471:13..471:23]: "BootVolSize" [plain @471:30..471:41] !<!Ref>
                  "VolumeType" [plain @472:13..472:23]: "BootVolType" [plain @472:30..472:41] !<!Ref>
            "SecurityGroupIds" [plain @473:7..473:23]: sequence [block]
              - "EC2InstanceSG" [plain @474:16..474:29] !<!Ref>
              - "ALBExternalAccessSG" [plain @475:16..475:35] !<!Ref>
            "KeyName" [plain @476:7..476:14]: "KeyPairName" [plain @476:21..476:32] !<!Ref>
            "Tags" [plain @477:7..477:11]: sequence [block]
              - mapping [block]
                "Key" [plain @478:11..478:14]: "Name" [plain @478:16..478:20]
                "Value" [plain @479:11..479:16]: "${AppName}-${Environment}-ec2-instance" [plain @479:23..479:61] !<!Sub>
              - mapping [block]
                "Key" [plain @480:11..480:14]: "Environment" [plain @480:16..480:27]
                "Value" [plain @481:11..481:16]: "Environment" [plain @481:23..481:34] !<!Ref>
        "EC2InstanceSG" [plain @484:3..484:16]: mapping [block]
          "Type" [plain @485:5..485:9]: "AWS::EC2::SecurityGroup" [plain @485:11..485:34]
          "Properties" [plain @486:5..486:15]: mapping [block]
            "GroupDescription" [plain @487:7..487:23]: "EC2 Instance Security Group" [plain @487:25..487:52]
            "VpcId" [plain @488:7..488:12]: "VpcId" [plain @488:19..488:24] !<!Ref>
            "Tags" [plain @489:7..489:11]: sequence [block]
              - mapping [block]
                "Key" [plain @490:11..490:14]: "Name" [plain @490:16..490:20]
                "Value" [plain @491:11..491:16]: "${AppName}-${Environment}-ec2-instance-SG" [plain @491:23..491:64] !<!Sub>
              - mapping [block]
                "Key" [plain @492:11..492:14]: "Environment" [plain @492:16..492:27]
                "Value" [plain @493:11..493:16]: "Environment" [plain @493:23..493:34] !<!Ref>
        "Tcp8080In" [plain @496:3..496:12]: mapping [block]
          "Type" [plain @497:5..497:9]: "AWS::EC2::SecurityGroupIngress" [plain @497:11..497:41]
          "Properties" [plain @498:5..498:15]: mapping [block]
            "GroupId" [plain @499:7..499:14]: "EC2InstanceSG" [plain @499:21..499:34] !<!Ref>
            "ToPort" [plain @500:7..500:13]: "8080" [double-quoted @500:15..500:21]
            "IpProtocol" [plain @501:7..501:17]: "tcp" [plain @501:19..501:22]
            "FromPort" [plain @502:7..502:15]: "8080" [double-quoted @502:17..502:23]
            "SourceSecurityGroupId" [plain @503:7..503:28]: "ALBExternalAccessSG" [plain @503:35..503:54] !<!Ref>
        "OriginALB" [plain @506:3..506:12]: mapping [block]
          "Type" [plain @507:5..507:9]: "AWS::ElasticLoadBalancingV2::LoadBalancer" [plain @507:11..507:52]
          "Properties" [plain @508:5..508:15]: mapping [block]
            "Name" [plain @509:7..509:11]: "${AppName}-${Environment}-alb" [plain @509:18..509:47] !<!Sub>
            "Scheme" [plain @510:7..510:13]: "ALBScheme" [plain @510:20..510:29] !<!Ref>
            "Type" [plain @511:7..511:11]: "ALBType" [plain @511:18..511:25] !<!Ref>
            "LoadBalancerAttributes" [plain @512:7..512:29]: sequence [block]
              - mapping [block]
                "Key" [plain @513:11..513:14]: "idle_timeout.timeout_seconds" [plain @513:16..513:44]
                "Value" [plain @514:11..514:16]: "ALBAttributeIdleTimeOut" [plain @514:23..514:46] !<!Ref>
              - mapping [block]
                "Key" [plain @515:11..515:14]: "deletion_protection.enabled" [plain @515:16..515:43]
                "Value" [plain @516:11..516:16]: "ALBAttributeDeletionProtection" [plain @516:23..516:53] !<!Ref>
              - mapping [block]
                "Key" [plain @517:11..517:14]: "routing.http2.enabled" [plain @517:16..517:37]
                "Value" [plain @518:11..518:16]: "ALBAttributeRoutingHttp2" [plain @518:23..518:47] !<!Ref>
            "Subnets" [plain @519:7..519:14]: sequence [block]
              - "PublicSubnetId1" [plain @520:16..520:31] !<!Ref>
              - "PublicSubnetId2" [plain @521:16..521:31] !<!Ref>
            "SecurityGroups" [plain @522:7..522:21]: sequence [block]
              - "ALBExternalAccessSG" [plain @523:16..523:35] !<!Ref>
            "Tags" [plain @524:7..524:11]: sequence [block]
              - mapping [block]
                "Key" [plain @525:11..525:14]: "Name" [plain @525:16..525:20]
                "Value" [plain @526:11..526:16]: "${AppName}-${Environment}-alb" [plain @526:23..526:52] !<!Sub>
              - mapping [block]
                "Key" [plain @527:11..527:14]: "Environment" [plain @527:16..527:27]
                "Value" [plain @528:11..528:16]: "Environment" [plain @528:23..528:34] !<!Ref>
        "OriginALBTG" [plain @531:3..531:14]: mapping [block]
          "Type" [plain @532:5..532:9]: "AWS::ElasticLoadBalancingV2::TargetGroup" [plain @532:11..532:51]
          "DependsOn" [plain @533:5..533:14]: "OriginALB" [plain @533:16..533:25]
          "Properties" [plain @534:5..534:15]: mapping [block]
            "Name" [plain @535:7..535:11]: "${AppName}-${Environment}-alb-tg" [plain @535:18..535:50] !<!Sub>
            "HealthCheckProtocol" [plain @536:7..536:26]: "HealthCheckProtocol" [plain @536:33..536:52] !<!Ref>
            "HealthCheckPath" [plain @537:7..537:22]: "HealthCheckPath" [plain @537:29..537:44] !<!Ref>
            "HealthCheckPort" [plain @538:7..538:22]: "${OriginALBTGPort}" [plain @538:29..538:47] !<!Sub>
            "HealthCheckIntervalSeconds" [plain @539:7..539:33]: "ALBTargetGroupHealthCheckIntervalSeconds" [plain @539:40..539:80] !<!Ref>
            "HealthCheckTimeoutSeconds" [plain @540:7..540:32]: "ALBTargetGroupHealthCheckTimeoutSeconds" [plain @540:39..540:78] !<!Ref>
            "HealthyThresholdCount" [plain @541:7..541:28]: "ALBTargetGroupHealthyThresholdCount" [plain @541:35..541:70] !<!Ref>
            "UnhealthyThresholdCount" [plain @542:7..542:30]: "ALBTargetGroupUnhealthyThresholdCount" [plain @542:37..542:74] !<!Ref>
            "TargetGroupAttributes" [plain @543:7..543:28]: sequence [block]
              - mapping [block]
                "Key" [plain @544:11..544:14]: "deregistration_delay.timeout_seconds" [plain @544:16..544:52]
                "Value" [plain @545:11..545:16]: "ALBTargetGroupAttributeDeregistration" [plain @545:23..545:60] !<!Ref>
            "TargetType" [plain @546:7..546:17]: "instance" [plain @546:19..546:27]
            "Targets" [plain @547:7..547:14]: sequence [block]
              - mapping [block]
                "Id" [plain @548:11..548:13]: "EC2Instance" [plain @548:20..548:31] !<!Ref>
                "Port" [plain @549:11..549:15]: "OriginALBTGPort" [plain @549:22..549:37] !<!Ref>
            "Port" [plain @550:7..550:11]: "OriginALBTGPort" [plain @550:18..550:33] !<!Ref>
            "Protocol" [plain @551:7..551:15]: "HTTP" [plain @551:17..551:21]
            "VpcId" [plain @552:7..552:12]: "VpcId" [plain @552:19..552:24] !<!Ref>
            "Tags" [plain @553:7..553:11]: sequence [block]
              - mapping [block]
                "Key" [plain @554:11..554:14]: "Name" [plain @554:16..554:20]
                "Value" [plain @555:11..555:16]: "${AppName}-${Environment}-alb-tg" [plain @555:23..555:55] !<!Sub>
              - mapping [block]
                "Key" [plain @556:11..556:14]: "Environment" [plain @556:16..556:27]
                "Value" [plain @557:11..557:16]: "Environment" [plain @557:23..557:34] !<!Ref>
        "OriginALBHttpsListener" [plain @560:3..560:25]: mapping [block]
          "Type" [plain @561:5..561:9]: "AWS::ElasticLoadBalancingV2::Listener" [plain @561:11..561:48]
          "DependsOn" [plain @562:5..562:14]: "OriginALBTG" [plain @562:16..562:27]
          "Properties" [plain @563:5..563:15]: mapping [block]
            "DefaultActions" [plain @564:7..564:21]: sequence [block]
              - mapping [block]
                "TargetGroupArn" [plain @565:11..565:25]: "OriginALBTG" [plain @565:32..565:43] !<!Ref>
                "Type" [plain @566:11..566:15]: "forward" [plain @566:17..566:24]
            "LoadBalancerArn" [plain @567:7..567:22]: "OriginALB" [plain @567:29..567:38] !<!Ref>
            "Port" [plain @568:7..568:11]: "443" [plain @568:13..568:16]
            "Protocol" [plain @569:7..569:15]: "HTTPS" [plain @569:17..569:22]
            "Certificates" [plain @570:7..570:19]: sequence [block]
              - mapping [block]
                "CertificateArn" [plain @571:11..571:25]: "arn:${AWS::Partition}:acm:${AWS::Region}:${AWS::AccountId}:certificate/${ACMCertificateIdentifier}" [plain @571:32..571:130] !<!Sub>
            "SslPolicy" [plain @572:7..572:16]: "ELBSecurityPolicy-FS-2018-06" [plain @572:18..572:46]
        "OriginALBHttpsListenerRule" [plain @575:3..575:29]: mapping [block]
          "Type" [plain @576:5..576:9]: "AWS::ElasticLoadBalancingV2::ListenerRule" [plain @576:11..576:52]
          "DependsOn" [plain @577:5..577:14]: "OriginALBHttpsListener" [plain @577:16..577:38]
          "Properties" [plain @578:5..578:15]: mapping [block]
            "Actions" [plain @579:7..579:14]: sequence [block]
              - mapping [block]
                "Type" [plain @580:11..580:15]: "forward" [plain @580:17..580:24]
                "TargetGroupArn" [plain @581:11..581:25]: "OriginALBTG" [plain @581:32..581:43] !<!Ref>
            "Conditions" [plain @582:7..582:17]: sequence [block]
              - mapping [block]
                "Field" [plain @583:11..583:16]: "path-pattern" [plain @583:18..583:30]
                "Values" [plain @584:11..584:17]: sequence [block]
                  - "/*" [plain @585:15..585:17]
            "ListenerArn" [plain @586:7..586:18]: "OriginALBHttpsListener" [plain @586:25..586:47] !<!Ref>
            "Priority" [plain @587:7..587:15]: "1" [plain @587:17..587:18]
        "ALBExternalAccessSG" [plain @590:3..590:22]: mapping [block]
          "Type" [plain @591:5..591:9]: "AWS::EC2::SecurityGroup" [plain @591:11..591:34]
          "Properties" [plain @592:5..592:15]: mapping [block]
            "GroupDescription" [plain @593:7..593:23]: "Allow external access to ALB" [plain @593:25..593:53]
            "VpcId" [plain @594:7..594:12]: "VpcId" [plain @594:19..594:24] !<!Ref>
            "Tags" [plain @595:7..595:11]: sequence [block]
              - mapping [block]
                "Key" [plain @596:11..596:14]: "Name" [plain @596:16..596:20]
                "Value" [plain @597:11..597:16]: "${AppName}-${Environment}-alb-external-access-ingrees-SG" [plain @597:23..597:79] !<!Sub>
              - mapping [block]
                "Key" [plain @598:11..598:14]: "Environment" [plain @598:16..598:27]
                "Value" [plain @599:11..599:16]: "Environment" [plain @599:23..599:34] !<!Ref>
        "HTTPSTcpIn" [plain @602:3..602:13]: mapping [block]
          "Type" [plain @603:5..603:9]: "AWS::EC2::SecurityGroupIngress" [plain @603:11..603:41]
          "Properties" [plain @604:5..604:15]: mapping [block]
            "GroupId" [plain @605:7..605:14]: "ALBExternalAccessSG" [plain @605:21..605:40] !<!Ref>
            "ToPort" [plain @606:7..606:13]: "443" [plain @606:15..606:18]
            "IpProtocol" [plain @607:7..607:17]: "tcp" [plain @607:19..607:22]
            "FromPort" [plain @608:7..608:15]: "443" [plain @608:17..608:20]
            "CidrIp" [plain @609:7..609:13]: "0.0.0.0/0" [plain @609:15..609:24]
        "HTTPTcpIn" [plain @611:3..611:12]: mapping [block]
          "Type" [plain @612:5..612:9]: "AWS::EC2::SecurityGroupIngress" [plain @612:11..612:41]
          "Properties" [plain @613:5..613:15]: mapping [block]
            "GroupId" [plain @614:7..614:14]: "ALBExternalAccessSG" [plain @614:21..614:40] !<!Ref>
            "ToPort" [plain @615:7..615:13]: "80" [plain @615:15..615:17]
            "IpProtocol" [plain @616:7..616:17]: "tcp" [plain @616:19..616:22]
            "FromPort" [plain @617:7..617:15]: "80" [plain @617:17..617:19]
            "CidrIp" [plain @618:7..618:13]: "0.0.0.0/0" [plain @618:15..618:24]
        "Tcp8080Out" [plain @621:3..621:13]: mapping [block]
          "Type" [plain @622:5..622:9]: "AWS::EC2::SecurityGroupEgress" [plain @622:11..622:40]
          "Properties" [plain @623:5..623:15]: mapping [block]
            "GroupId" [plain @624:7..624:14]: "ALBExternalAccessSG" [plain @624:21..624:40] !<!Ref>
            "ToPort" [plain @625:7..625:13]: "8080" [plain @625:15..625:19]
            "IpProtocol" [plain @626:7..626:17]: "tcp" [plain @626:19..626:22]
            "FromPort" [plain @627:7..627:15]: "8080" [plain @627:17..627:21]
            "DestinationSecurityGroupId" [plain @628:7..628:33]: "EC2InstanceSG" [plain @628:40..628:53] !<!Ref>
        "CloudFrontDistribution" [plain @631:3..631:25]: mapping [block]
          "Type" [plain @632:5..632:9]: "AWS::CloudFront::Distribution" [plain @632:11..632:40]
          "DependsOn" [plain @633:5..633:14]: sequence [block]
            - "LoggingBucket" [plain @634:9..634:22]
            - "LambdaEdgeFunction" [plain @635:9..635:27]
          "Properties" [plain @636:5..636:15]: mapping [block]
            "DistributionConfig" [plain @637:7..637:25]: mapping [block]
              "Comment" [plain @638:9..638:16]: "Cloudfront Distribution pointing ALB Origin" [plain @638:18..638:61]
              "Origins" [plain @639:9..639:16]: sequence [block]
                - mapping [block]
                  "DomainName" [plain @640:13..640:23]: "OriginALB.DNSName" [plain @640:33..640:50] !<!GetAtt>
                  "Id" [plain @641:13..641:15]: "OriginALB" [plain @641:22..641:31] !<!Ref>
                  "CustomOriginConfig" [plain @642:13..642:31]: mapping [block]
                    "HTTPPort" [plain @643:15..643:23]: "80" [plain @643:25..643:27]
                    "HTTPSPort" [plain @644:15..644:24]: "443" [plain @644:26..644:29]
                    "OriginProtocolPolicy" [plain @645:15..645:35]: "OriginProtocolPolicy" [plain @645:42..645:62] !<!Ref>
                    "OriginKeepaliveTimeout" [plain @646:15..646:37]: "OriginKeepaliveTimeout" [plain @646:44..646:66] !<!Ref>
                    "OriginReadTimeout" [plain @647:15..647:32]: "OriginReadTimeout" [plain @647:39..647:56] !<!Ref>
                    "OriginSSLProtocols" [plain @648:15..648:33]: sequence [block]
                      - "TLSv1" [plain @649:19..649:24]
                      - "TLSv1.1" [plain @650:19..650:26]
                      - "TLSv1.2" [plain @651:19..651:26]
                      - "SSLv3" [plain @652:19..652:24]
              "Enabled" [plain @653:9..653:16]: "true" [plain @653:18..653:22]
              "HttpVersion" [plain @654:9..654:20]: "http2" [plain @654:22..654:27]
              "Aliases" [plain @655:9..655:16]: sequence [block]
                - "AlternateDomainNames" [plain @656:18..656:38] !<!Ref>
              "DefaultCacheBehavior" [plain @657:9..657:29]: mapping [block]
                "AllowedMethods" [plain @658:11..658:25]: sequence [block]
                  - "GET" [plain @659:15..659:18]
                  - "HEAD" [plain @660:15..660:19]
                  - "DELETE" [plain @661:15..661:21]
                  - "OPTIONS" [plain @662:15..662:22]
                  - "PATCH" [plain @663:15..663:20]
                  - "POST" [plain @664:15..664:19]
                  - "PUT" [plain @665:15..665:18]
                "Compress" [plain @666:11..666:19]: "Compress" [plain @666:26..666:34] !<!Ref>
                "DefaultTTL" [plain @667:11..667:21]: "DefaultTTL" [plain @667:28..667:38] !<!Ref>
                "MaxTTL" [plain @668:11..668:17]: "MaxTTL" [plain @668:24..668:30] !<!Ref>
                "MinTTL" [plain @669:11..669:17]: "MinTTL" [plain @669:24..669:30] !<!Ref>
                "SmoothStreaming" [plain @670:11..670:26]: "false" [double-quoted @670:28..670:35]
                "TargetOriginId" [plain @671:11..671:25]: "OriginALB" [plain @671:32..671:41] !<!Ref>
                "ForwardedValues" [plain @672:11..672:26]: mapping [block]
                  "QueryString" [plain @673:13..673:24]: "QueryString" [plain @673:31..673:42] !<!Ref>
                  "Cookies" [plain @674:13..674:20]: mapping [block]
                    "Forward" [plain @675:15..675:22]: "ForwardCookies" [plain @675:29..675:43] !<!Ref>
                "ViewerProtocolPolicy" [plain @676:11..676:31]: "ViewerProtocolPolicy" [plain @676:38..676:58] !<!Ref>
                "LambdaFunctionAssociations" [plain @677:11..677:37]: sequence [block]
                  - mapping [block]
                    "EventType" [plain @678:15..678:24]: "LambdaEventType" [plain @678:31..678:46] !<!Ref>
                    "LambdaFunctionARN" [plain @679:15..679:32]: "LambdaEdgeVersion" [plain @679:39..679:56] !<!Ref>
              "PriceClass" [plain @680:9..680:19]: "PriceClass" [plain @680:26..680:36] !<!Ref>
              "ViewerCertificate" [plain @681:9..681:26]: mapping [block]
                "AcmCertificateArn" [plain @682:11..682:28]: "arn:${AWS::Partition}:acm:${AWS::Region}:${AWS::AccountId}:certificate/${ACMCertificateIdentifier}" [plain @682:35..682:133] !<!Sub>
                "SslSupportMethod" [plain @683:11..683:27]: "SslSupportMethod" [plain @683:34..683:50] !<!Ref>
                "MinimumProtocolVersion" [plain @684:11..684:33]: "MinimumProtocolVersion" [plain @684:40..684:62] !<!Ref>
              "IPV6Enabled" [plain @685:9..685:20]: "IPV6Enabled" [plain @685:27..685:38] !<!Ref>
              "Logging" [plain @686:9..686:16]: mapping [block]
                "Bucket" [plain @687:11..687:17]: "${LoggingBucket}.s3.amazonaws.com" [plain @687:24..687:57] !<!Sub>
        "LambdaEdgeFunction" [plain @690:3..690:21]: mapping [block]
          "Type" [plain @691:5..691:9]: "AWS::Lambda::Function" [plain @691:11..691:32]
          "Metadata" [plain @692:5..692:13]: mapping [block]
            "guard" [plain @693:7..693:12]: mapping [block]
              "SuppressedRules" [plain @694:9..694:24]: sequence [block]
                - "LAMBDA_INSIDE_VPC" [plain @695:13..695:30]
          "Properties" [plain @696:5..696:15]: mapping [block]
            "Description" [plain @697:7..697:18]: "A custom Lambda@Edge function for serving custom headers from CloudFront Distribution" [plain @697:20..697:105]
            "FunctionName" [plain @698:7..698:19]: "${AppName}-lambda-edge-${Environment}" [plain @698:26..698:63] !<!Sub>
            "Handler" [plain @699:7..699:14]: "index.handler" [plain @699:16..699:29]
            "Role" [plain @700:7..700:11]: "LambdaEdgeIAMRole.Arn" [plain @700:21..700:42] !<!GetAtt>
            "MemorySize" [plain @701:7..701:17]: "128" [plain @701:19..701:22]
            "Timeout" [plain @702:7..702:14]: "5" [plain @702:16..702:17]
            "Code" [plain @703:7..703:11]: mapping [block]
              "ZipFile" [plain @704:9..704:16]: literal [@704:18..737:1]
                "'use strict';\n"
                "\n"
                " exports.handler = (event, context, callback) => {\n"
                "    console.log('Adding additional headers to CloudFront response.');\n"
                "\n"
                "    const response = event.Records[0].cf.response;\n"
                "    response.headers['strict-transport-security'] = [{\n"
                "    key: 'Strict-Transport-Security',\n"
                "    value: 'max-age=86400; includeSubdomains; preload',\n"
                "    }];\n"
                "    response.headers['x-content-type-options'] = [{\n"
                "    key: 'X-Content-Type-Options',\n"
                "    value: 'nosniff',\n"
                "    }];\n"
                "    response.headers['x-frame-options'] = [{\n"
                "        key:   'X-Frame-Options',\n"
                "        value: \"DENY\"\n"
                "    }];\n"
                "    response.headers['content-security-policy'] = [{\n"
                "        key:   'Content-Security-Policy',\n"
                "        value: \"default-src 'none'; img-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'\"\n"
                "    }];\n"
                "    response.headers['x-xss-protection'] = [{\n"
                "        key:   'X-XSS-Protection',\n"
                "        value: \"1; mode=block\"\n"
                "    }];\n"
                "    response.headers['referrer-policy'] = [{\n"
                "        key:   'Referrer-Policy',\n"
                "        value: \"same-origin\"\n"
                "    }];\n"
                "    callback(null, response);\n"
                "  };\n"
            "Runtime" [plain @737:7..737:14]: "nodejs20.x" [plain @737:16..737:26]
        "LambdaEdgeVersion" [plain @739:3..739:20]: mapping [block]
          "Type" [plain @740:5..740:9]: "AWS::Lambda::Version" [plain @740:11..740:31]
          "Properties" [plain @741:5..741:15]: mapping [block]
            "FunctionName" [plain @742:7..742:19]: "LambdaEdgeFunction" [plain @742:26..742:44] !<!Ref>
      "Outputs" [plain @744:1..744:8]: mapping [block]
        "AdministratorAccessIAMRole" [plain @745:3..745:29]: mapping [block]
          "Description" [plain @746:5..746:16]: "Administrator Access IAM Role" [plain @746:18..746:47]
          "Value" [plain @747:5..747:10]: "AdministratorAccessIAMRole" [plain @747:17..747:43] !<!Ref>
          "Export" [plain @748:5..748:11]: mapping [block]
            "Name" [plain @749:7..749:11]: "${AppName}-iam-${Environment}-administrator-access-role" [plain @749:18..749:73] !<!Sub>
        "LoggingBucket" [plain @751:3..751:16]: mapping [block]
          "Description" [plain @752:5..752:16]: "Name of S3 Logging bucket" [plain @752:18..752:43]
          "Value" [plain @753:5..753:10]: "LoggingBucket" [plain @753:17..753:30] !<!Ref>
          "Export" [plain @754:5..754:11]: mapping [block]
            "Name" [plain @755:7..755:11]: "${AppName}-logging-${Environment}-${AWS::AccountId}-${AWS::Region}" [plain @755:18..755:84] !<!Sub>
        "LoggingBucketKMSKey" [plain @757:3..757:22]: mapping [block]
          "Description" [plain @758:5..758:16]: "Logging Bucket KMS Key" [plain @758:18..758:40]
          "Value" [plain @759:5..759:10]: "LoggingBucketKMSKey" [plain @759:17..759:36] !<!Ref>
          "Export" [plain @760:5..760:11]: mapping [block]
            "Name" [plain @761:7..761:11]: "${AppName}-${Environment}-s3-logging-kms" [plain @761:18..761:58] !<!Sub>
        "OriginALB" [plain @763:3..763:12]: mapping [block]
          "Description" [plain @764:5..764:16]: "The URL of the Origin ALB" [plain @764:18..764:43]
          "Value" [plain @765:5..765:10]: "OriginALB.DNSName" [plain @765:20..765:37] !<!GetAtt>
          "Export" [plain @766:5..766:11]: mapping [block]
            "Name" [plain @767:7..767:11]: "${AppName}-${Environment}-origin-alb-dns" [plain @767:18..767:58] !<!Sub>
        "ALBExternalAccessSGID" [plain @769:3..769:24]: mapping [block]
          "Description" [plain @770:5..770:16]: "ALB External Access Security Group ID" [plain @770:18..770:55]
          "Value" [plain @771:5..771:10]: "ALBExternalAccessSG" [plain @771:17..771:36] !<!Ref>
          "Export" [plain @772:5..772:11]: mapping [block]
            "Name" [plain @773:7..773:11]: "${AppName}-${Environment}-alb-external-access-ingrees-sg" [plain @773:18..773:74] !<!Sub>
        "EC2InstanceSGID" [plain @775:3..775:18]: mapping [block]
          "Description" [plain @776:5..776:16]: "EC2 Instance Security Group ID" [plain @776:18..776:48]
          "Value" [plain @777:5..777:10]: "EC2InstanceSG" [plain @777:17..777:30] !<!Ref>
          "Export" [plain @778:5..778:11]: mapping [block]
            "Name" [plain @779:7..779:11]: "${AppName}-${Environment}-ec2-instance-sg" [plain @779:18..779:59] !<!Sub>
        "EC2InstanceDNS" [plain @781:3..781:17]: mapping [block]
          "Description" [plain @782:5..782:16]: "EC2 Instance DNS Name" [plain @782:18..782:39]
          "Value" [plain @783:5..783:10]: "EC2Instance.PrivateDnsName" [plain @783:20..783:46] !<!GetAtt>
          "Export" [plain @784:5..784:11]: mapping [block]
            "Name" [plain @785:7..785:11]: "${AppName}-${Environment}-ec2-instance-dns" [plain @785:18..785:60] !<!Sub>
        "EC2InstanceIP" [plain @787:3..787:16]: mapping [block]
          "Description" [plain @788:5..788:16]: "EC2 Instance IP Address" [plain @788:18..788:41]
          "Value" [plain @789:5..789:10]: "EC2Instance.PrivateIp" [plain @789:20..789:41] !<!GetAtt>
          "Export" [plain @790:5..790:11]: mapping [block]
            "Name" [plain @791:7..791:11]: "${AppName}-${Environment}-ec2-instance-ip-address" [plain @791:18..791:67] !<!Sub>
        "EC2InstanceID" [plain @793:3..793:16]: mapping [block]
          "Description" [plain @794:5..794:16]: "EC2 Instance Instance ID" [plain @794:18..794:42]
          "Value" [plain @795:5..795:10]: "EC2Instance" [plain @795:17..795:28] !<!Ref>
          "Export" [plain @796:5..796:11]: mapping [block]
            "Name" [plain @797:7..797:11]: "${AppName}-${Environment}-ec2-instance-id" [plain @797:18..797:59] !<!Sub>
        "CloudFrontEndpoint" [plain @799:3..799:21]: mapping [block]
          "Description" [plain @800:5..800:16]: "Endpoint for Cloudfront Distribution" [plain @800:18..800:54]
          "Value" [plain @801:5..801:10]: "CloudFrontDistribution" [plain @801:17..801:39] !<!Ref>
          "Export" [plain @802:5..802:11]: mapping [block]
            "Name" [plain @803:7..803:11]: "${AppName}-${Environment}-cloudfront-distribution" [plain @803:18..803:67] !<!Sub>
        "AlternateDomainNames" [plain @805:3..805:23]: mapping [block]
          "Description" [plain @806:5..806:16]: "Alternate Domain Names (CNAME)" [plain @806:18..806:48]
          "Value" [plain @807:5..807:10]: "AlternateDomainNames" [plain @807:17..807:37] !<!Ref>
        "LambdaEdgeFunction" [plain @809:3..809:21]: mapping [block]
          "Description" [plain @810:5..810:16]: "The Name of the Lambda@Edge Function" [plain @810:18..810:54]
          "Value" [plain @811:5..811:10]: "LambdaEdgeFunction" [plain @811:12..811:30]
          "Export" [plain @812:5..812:11]: mapping [block]
            "Name" [plain @813:7..813:11]: "${AppName}-${Environment}-lambda-edge-function-3" [plain @813:18..813:66] !<!Sub>
        "LambdaEdgeFunctionARN" [plain @815:3..815:24]: mapping [block]
          "Description" [plain @816:5..816:16]: "The ARN of the Lambda@Edge Function" [plain @816:18..816:53]
          "Value" [plain @817:5..817:10]: "LambdaEdgeFunction.Arn" [plain @817:20..817:42] !<!GetAtt>
          "Export" [plain @818:5..818:11]: mapping [block]
            "Name" [plain @819:7..819:11]: "${AppName}-${Environment}-lambda-edge-function-arn-3" [plain @819:18..819:70] !<!Sub>
        "LambdaEdgeVersion" [plain @821:3..821:20]: mapping [block]
          "Description" [plain @822:5..822:16]: "Lambda@Edge Version Function" [plain @822:18..822:46]
          "Value" [plain @823:5..823:10]: "LambdaEdgeVersion.Version" [plain @823:20..823:45] !<!GetAtt>
    "#);
}
