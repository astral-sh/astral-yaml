#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-neptunedb-neptune-04ad27df"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "AWS CloudFormation template to automatically provision AWS Neptune Graph Database through optimized CI/CD method with AWS CloudWatch and AWS SNS." [plain @3:14..3:159]
      "Metadata" [plain @5:1..5:9]: mapping [block]
        "cfn-lint" [plain @6:3..6:11]: mapping [block]
          "config" [plain @7:5..7:11]: mapping [block]
            "regions" [plain @8:7..8:14]: sequence [block]
              - "us-east-1" [plain @9:11..9:20]
              - "us-west-2" [plain @10:11..10:20]
        "AWS::CloudFormation::Interface" [plain @12:3..12:33]: mapping [block]
          "ParameterGroups" [plain @13:5..13:20]: sequence [block]
            - mapping [block]
              "Label" [plain @14:9..14:14]: mapping [block]
                "default" [plain @15:11..15:18]: "Tags" [plain @15:20..15:24]
              "Parameters" [plain @16:9..16:19]: sequence [block]
                - "Env" [plain @17:13..17:16]
                - "AppName" [plain @18:13..18:20]
                - "User" [plain @19:13..19:17]
                - "Owner" [plain @20:13..20:18]
                - "Tier" [plain @21:13..21:17]
                - "Version" [plain @22:13..22:20]
                - "Storage" [plain @23:13..23:20]
            - mapping [block]
              "Label" [plain @24:9..24:14]: mapping [block]
                "default" [plain @25:11..25:18]: "Neptune Database Parameters" [plain @25:20..25:47]
              "Parameters" [plain @26:9..26:19]: sequence [block]
                - "DBInstanceClass" [plain @27:13..27:28]
                - "DBClusterIdentifier" [plain @28:13..28:32]
                - "MajorVersionUpgrade" [plain @29:13..29:32]
                - "MinorVersionUpgrade" [plain @30:13..30:32]
                - "NeptuneQueryTimeout" [plain @31:13..31:32]
                - "StorageEncrypted" [plain @32:13..32:29]
                - "NeptuneDBSubnetGroupName" [plain @33:13..33:37]
                - "GremlinRequestsPerSecThreshold" [plain @34:13..34:43]
                - "SparqlRequestsPerSecThreshold" [plain @35:13..35:42]
                - "IAMAuthEnabled" [plain @36:13..36:27]
                - "VPCStack" [plain @37:13..37:21]
            - mapping [block]
              "Label" [plain @38:9..38:14]: mapping [block]
                "default" [plain @39:11..39:18]: "Neptune Security Group Parameters" [plain @39:20..39:53]
              "Parameters" [plain @40:9..40:19]: sequence [block]
                - "Port" [plain @41:13..41:17]
            - mapping [block]
              "Label" [plain @42:9..42:14]: mapping [block]
                "default" [plain @43:11..43:18]: "Neptune Backup and Monitoring Parameters" [plain @43:20..43:60]
              "Parameters" [plain @44:9..44:19]: sequence [block]
                - "HighCpuAlarmThreshold" [plain @45:13..45:34]
                - "LowMemoryAlarmThreshold" [plain @46:13..46:36]
                - "UploadAuditLogs" [plain @47:13..47:28]
                - "BackupRetentionPeriod" [plain @48:13..48:34]
                - "NeptuneDBClusterPreferredMaintenanceWindow" [plain @49:13..49:55]
                - "NeptuneDBInstancePreferredMaintenanceWindow" [plain @50:13..50:56]
                - "NeptuneDBClusterPreferredBackupWindow" [plain @51:13..51:50]
            - mapping [block]
              "Label" [plain @52:9..52:14]: mapping [block]
                "default" [plain @53:11..53:18]: "Neptune Notifications Parameters" [plain @53:20..53:52]
              "Parameters" [plain @54:9..54:19]: sequence [block]
                - "NeptuneSNSTopicArn" [plain @55:13..55:31]
                - "SNSEmailSubscription" [plain @56:13..56:33]
      "Parameters" [plain @58:1..58:11]: mapping [block]
        "Env" [plain @61:3..61:6]: mapping [block]
          "Description" [plain @62:5..62:16]: "Please specify the target Environment. Used for tagging and resource names. Mandatory LOWER CASE." [plain @62:18..62:115]
          "Type" [plain @63:5..63:9]: "String" [plain @63:11..63:17]
          "AllowedValues" [plain @64:5..64:18]: sequence [block]
            - "test" [plain @65:9..65:13]
            - "dev" [plain @66:9..66:12]
            - "prod" [plain @67:9..67:13]
          "Default" [plain @70:5..70:12]: "dev" [plain @70:14..70:17]
        "AppName" [plain @72:3..72:10]: mapping [block]
          "Description" [plain @73:5..73:16]: "Please specify the Application Name. Used for tagging and resource names. Mandatory LOWER CASE." [plain @73:18..73:113]
          "Type" [plain @74:5..74:9]: "String" [plain @74:11..74:17]
          "Default" [plain @75:5..75:12]: "appname" [plain @75:14..75:21]
        "User" [plain @78:3..78:7]: mapping [block]
          "Description" [plain @79:5..79:16]: "Please specify the User. Used for tagging" [plain @79:18..79:59]
          "Type" [plain @80:5..80:9]: "String" [plain @80:11..80:17]
          "Default" [plain @81:5..81:12]: "test" [plain @81:14..81:18]
        "Owner" [plain @84:3..84:8]: mapping [block]
          "Description" [plain @85:5..85:16]: "Please specify the Owner. Used for tagging" [plain @85:18..85:60]
          "Type" [plain @86:5..86:9]: "String" [plain @86:11..86:17]
          "Default" [plain @87:5..87:12]: "" [double-quoted @87:14..87:16]
        "Tier" [plain @90:3..90:7]: mapping [block]
          "Description" [plain @91:5..91:16]: "Please specify the Tier. Used for tagging" [plain @91:18..91:59]
          "Type" [plain @92:5..92:9]: "String" [plain @92:11..92:17]
          "Default" [plain @93:5..93:12]: "" [double-quoted @93:14..93:16]
        "Version" [plain @96:3..96:10]: mapping [block]
          "Description" [plain @97:5..97:16]: "Please specify the Application Version. Used for tagging" [plain @97:18..97:74]
          "Type" [plain @98:5..98:9]: "String" [plain @98:11..98:17]
          "Default" [plain @99:5..99:12]: "" [double-quoted @99:14..99:16]
        "Storage" [plain @102:3..102:10]: mapping [block]
          "Description" [plain @103:5..103:16]: "Please specify the Storage Type. Used for tagging" [plain @103:18..103:67]
          "Type" [plain @104:5..104:9]: "String" [plain @104:11..104:17]
          "AllowedValues" [plain @105:5..105:18]: sequence [block]
            - "ebs" [plain @106:9..106:12]
            - "efs" [plain @107:9..107:12]
            - "s3" [plain @108:9..108:11]
          "Default" [plain @111:5..111:12]: "ebs" [plain @111:14..111:17]
        "VPCStack" [plain @113:3..113:11]: mapping [block]
          "Description" [plain @114:5..114:16]: "Please specify the VPC Stack Name." [plain @114:18..114:52]
          "Type" [plain @115:5..115:9]: "String" [plain @115:11..115:17]
          "Default" [plain @116:5..116:12]: "vpc" [plain @116:14..116:17]
        "DBInstanceClass" [plain @119:3..119:18]: mapping [block]
          "Description" [plain @120:5..120:16]: "Neptune DB instance class that will be used for primary and all replicas" [plain @120:18..120:90]
          "Type" [plain @121:5..121:9]: "String" [plain @121:11..121:17]
          "AllowedValues" [plain @122:5..122:18]: sequence [block]
            - "db.r4.large" [plain @123:9..123:20]
            - "db.r4.xlarge" [plain @124:9..124:21]
            - "db.r4.2xlarge" [plain @125:9..125:22]
            - "db.r4.4xlarge" [plain @126:9..126:22]
            - "db.r4.8xlarge" [plain @127:9..127:22]
          "Default" [plain @128:5..128:12]: "db.r4.large" [plain @128:14..128:25]
        "DBClusterIdentifier" [plain @131:3..131:22]: mapping [block]
          "Description" [plain @132:5..132:16]: "Neptune DB cluster identifier. Must contain from 1 to 63 alphanumeric characters or hyphens. First character must be a letter. Cannot end with a hyphen or contain two consecutive hyphens." [plain @132:18..132:205]
          "Type" [plain @133:5..133:9]: "String" [plain @133:11..133:17]
        "Port" [plain @142:3..142:7]: mapping [block]
          "Description" [plain @143:5..143:16]: "Port used to connect to the Neptune cluster. Must be a valid port number between" [plain @143:18..143:98]
          "Type" [plain @144:5..144:9]: "String" [plain @144:11..144:17]
          "Default" [plain @145:5..145:12]: "8182" [plain @145:14..145:18]
        "NeptuneQueryTimeout" [plain @148:3..148:22]: mapping [block]
          "Description" [plain @149:5..149:16]: "Neptune DB parameters. Allowed values \"10-2147483647\"" [plain @149:18..149:71]
          "Type" [plain @150:5..150:9]: "String" [plain @150:11..150:17]
          "Default" [plain @151:5..151:12]: "120000" [plain @151:14..151:20]
        "StorageEncrypted" [plain @154:3..154:19]: mapping [block]
          "Description" [plain @155:5..155:16]: "Data-at-rest encryption" [plain @155:18..155:41]
          "Type" [plain @156:5..156:9]: "String" [plain @156:11..156:17]
          "AllowedValues" [plain @157:5..157:18]: sequence [block]
            - "true" [plain @158:9..158:13]
            - "false" [plain @159:9..159:14]
          "Default" [plain @160:5..160:12]: "false" [plain @160:14..160:19]
        "NeptuneDBSubnetGroupName" [plain @163:3..163:27]: mapping [block]
          "Description" [plain @164:5..164:16]: "The name for the DB Subnet Group. This value is stored as a lowercase string. Constraints, Must contain no more than 255 letters, numbers, periods, underscores, spaces, or hyphens. Must not be default." [plain @164:18..164:219]
          "Type" [plain @165:5..165:9]: "String" [plain @165:11..165:17]
        "NeptuneSNSTopicArn" [plain @168:3..168:21]: mapping [block]
          "Description" [plain @169:5..169:16]: "Custom SNS topic alarm. Optional. If not provided, an new SNS topic will be created for you" [plain @169:18..169:109]
          "Type" [plain @170:5..170:9]: "String" [plain @170:11..170:17]
        "SNSEmailSubscription" [plain @173:3..173:23]: mapping [block]
          "Description" [plain @174:5..174:16]: "SNS Email subscription. Optional. If not provided, no alarm subscriptions will be created" [plain @174:18..174:107]
          "Type" [plain @175:5..175:9]: "String" [plain @175:11..175:17]
          "AllowedPattern" [plain @176:5..176:19]: "^[\\w-\\+]+(\\.[\\w]+)*@[\\w-]+(\\.[\\w]+)*(\\.[a-z]{2,})$|^$" [plain @176:21..176:74]
        "HighCpuAlarmThreshold" [plain @179:3..179:24]: mapping [block]
          "Description" [plain @180:5..180:16]: "High CPU alarm threshold. Alert when CPU goes above this value.  In percentage used" [plain @180:18..180:101]
          "Type" [plain @181:5..181:9]: "String" [plain @181:11..181:17]
          "Default" [plain @182:5..182:12]: "80" [plain @182:14..182:16]
          "AllowedPattern" [plain @183:5..183:19]: "(100|[1-9]?[0-9])$" [plain @183:21..183:39]
        "LowMemoryAlarmThreshold" [plain @186:3..186:26]: mapping [block]
          "Description" [plain @187:5..187:16]: "Low memory alarm threshold. Alert when memory falls below this value.  In bytes" [plain @187:18..187:97]
          "Type" [plain @188:5..188:9]: "Number" [plain @188:11..188:17]
          "Default" [plain @189:5..189:12]: "700000000" [plain @189:14..189:23]
        "GremlinRequestsPerSecThreshold" [plain @192:3..192:33]: mapping [block]
          "Description" [plain @193:5..193:16]: "Gremlin Requests Per Sec alarm threshold. Alert when Gremlin Requests Per Sec goes above this value. In percentage used" [plain @193:18..193:137]
          "Type" [plain @194:5..194:9]: "String" [plain @194:11..194:17]
          "Default" [plain @195:5..195:12]: "10000" [plain @195:14..195:19]
        "SparqlRequestsPerSecThreshold" [plain @198:3..198:32]: mapping [block]
          "Description" [plain @199:5..199:16]: "Sparql Requests Per Sec alarm threshold. Alert when Sparql Requests Per Sec goes above this value. In percentage used" [plain @199:18..199:135]
          "Type" [plain @200:5..200:9]: "String" [plain @200:11..200:17]
          "Default" [plain @201:5..201:12]: "10000" [plain @201:14..201:19]
        "UploadAuditLogs" [plain @204:3..204:18]: mapping [block]
          "Description" [plain @205:5..205:16]: "Enable upload of audit logs?" [plain @205:18..205:46]
          "Type" [plain @206:5..206:9]: "String" [plain @206:11..206:17]
          "AllowedValues" [plain @207:5..207:18]: sequence [block]
            - "Yes" [plain @208:9..208:12]
            - "No" [plain @209:9..209:11]
          "Default" [plain @210:5..210:12]: "Yes" [plain @210:14..210:17]
        "BackupRetentionPeriod" [plain @213:3..213:24]: mapping [block]
          "Description" [plain @214:5..214:16]: "Backup retention period (in days).  Must be between 1 - 35" [plain @214:18..214:76]
          "Type" [plain @215:5..215:9]: "String" [plain @215:11..215:17]
          "Default" [plain @216:5..216:12]: "31" [plain @216:14..216:16]
          "AllowedPattern" [plain @217:5..217:19]: "([1-9]|[12][0-9]|3[0-5])" [plain @217:21..217:45]
        "NeptuneDBClusterPreferredMaintenanceWindow" [plain @220:3..220:45]: mapping [block]
          "Description" [plain @221:5..221:16]: "Neptune DB cluster preferred maintenance window. Format - ddd:hh24:mi-ddd:hh24:mi. Valid Days - Mon, Tue, Wed, Thu, Fri, Sat, Sun. Constraints - Minimum 30-minute window." [plain @221:18..221:188]
          "Type" [plain @222:5..222:9]: "String" [plain @222:11..222:17]
          "Default" [plain @223:5..223:12]: "mon:03:00-mon:04:00" [plain @223:14..223:33]
        "NeptuneDBInstancePreferredMaintenanceWindow" [plain @226:3..226:46]: mapping [block]
          "Description" [plain @227:5..227:16]: "Neptune DB instance preferred maintenance window. Format - ddd:hh24:mi-ddd:hh24:mi. Valid Days - Mon, Tue, Wed, Thu, Fri, Sat, Sun. Constraints - Minimum 30-minute window." [plain @227:18..227:189]
          "Type" [plain @228:5..228:9]: "String" [plain @228:11..228:17]
          "Default" [plain @229:5..229:12]: "mon:03:00-mon:04:00" [plain @229:14..229:33]
        "NeptuneDBClusterPreferredBackupWindow" [plain @232:3..232:40]: mapping [block]
          "Description" [plain @233:5..233:16]: "Neptune DB cluster preferred backup window. Constrains - Must be in the format hh24:mi-hh24:mi. Must be in Universal Coordinated Time (UTC). Must not conflict with the preferred maintenance window. Must be at least 30 minutes." [plain @233:18..233:244]
          "Type" [plain @234:5..234:9]: "String" [plain @234:11..234:17]
          "Default" [plain @235:5..235:12]: "02:00-03:00" [plain @235:14..235:25]
        "MajorVersionUpgrade" [plain @238:3..238:22]: mapping [block]
          "Description" [plain @239:5..239:16]: "Neptune DB major version upgrade" [plain @239:18..239:50]
          "Type" [plain @240:5..240:9]: "String" [plain @240:11..240:17]
          "AllowedValues" [plain @241:5..241:18]: sequence [block]
            - "true" [plain @242:9..242:13]
            - "false" [plain @243:9..243:14]
          "Default" [plain @244:5..244:12]: "true" [plain @244:14..244:18]
        "MinorVersionUpgrade" [plain @247:3..247:22]: mapping [block]
          "Description" [plain @248:5..248:16]: "Neptune DB minor version upgrade" [plain @248:18..248:50]
          "Type" [plain @249:5..249:9]: "String" [plain @249:11..249:17]
          "AllowedValues" [plain @250:5..250:18]: sequence [block]
            - "true" [plain @251:9..251:13]
            - "false" [plain @252:9..252:14]
          "Default" [plain @253:5..253:12]: "true" [plain @253:14..253:18]
        "IAMAuthEnabled" [plain @256:3..256:17]: mapping [block]
          "Description" [plain @257:5..257:16]: "Neptune DB IAM authentication" [plain @257:18..257:47]
          "Type" [plain @258:5..258:9]: "String" [plain @258:11..258:17]
          "AllowedValues" [plain @259:5..259:18]: sequence [block]
            - "true" [plain @260:9..260:13]
            - "false" [plain @261:9..261:14]
          "Default" [plain @262:5..262:12]: "false" [plain @262:14..262:19]
      "Conditions" [plain @264:1..264:11]: mapping [block]
        "EnableAuditLogUpload" [plain @267:3..267:23]: sequence [block] !<!Equals>
          - "UploadAuditLogs" [plain @268:12..268:27] !<!Ref>
          - "Yes" [plain @269:7..269:10]
        "CreateSnsTopic" [plain @272:3..272:17]: sequence [block] !<!Equals>
          - "NeptuneSNSTopicArn" [plain @273:12..273:30] !<!Ref>
          - "" [double-quoted @274:7..274:9]
        "CreateSnsSubscription" [plain @277:3..277:24]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "SNSEmailSubscription" [plain @279:14..279:34] !<!Ref>
            - "" [double-quoted @280:9..280:11]
      "Resources" [plain @282:1..282:10]: mapping [block]
        "NeptuneDBSG" [plain @283:3..283:14]: mapping [block]
          "Type" [plain @284:5..284:9]: "AWS::EC2::SecurityGroup" [plain @284:11..284:34]
          "Properties" [plain @285:5..285:15]: mapping [block]
            "GroupDescription" [plain @286:7..286:23]: "SG of Neptune DB" [plain @286:25..286:41]
            "VpcId" [plain @287:7..287:12]: mapping [block] !<!ImportValue>
              "Fn::Sub" [plain @288:9..288:16]: "${VPCStack}-VPCID" [plain @288:18..288:35]
            "Tags" [plain @289:7..289:11]: sequence [block]
              - mapping [block]
                "Key" [plain @290:11..290:14]: "Name" [plain @290:16..290:20]
                "Value" [plain @291:11..291:16]: "${AWS::StackName}-neptune-sg" [plain @291:23..291:51] !<!Sub>
        "NeptuneDBCluster" [plain @295:3..295:19]: mapping [block]
          "Type" [plain @296:5..296:9]: "AWS::Neptune::DBCluster" [plain @296:11..296:34]
          "DependsOn" [plain @297:5..297:14]: "NeptuneDBSG" [plain @297:16..297:27]
          "Properties" [plain @298:5..298:15]: mapping [block]
            "BackupRetentionPeriod" [plain @301:7..301:28]: "BackupRetentionPeriod" [plain @301:35..301:56] !<!Ref>
            "DBClusterIdentifier" [plain @302:7..302:26]: "DBClusterIdentifier" [plain @302:33..302:52] !<!Ref>
            "DBClusterParameterGroupName" [plain @303:7..303:34]: "NeptuneDBClusterParameterGroup" [plain @303:41..303:71] !<!Ref>
            "DBSubnetGroupName" [plain @304:7..304:24]: "NeptuneDBSubnetGroup" [plain @304:31..304:51] !<!Ref>
            "IamAuthEnabled" [plain @305:7..305:21]: "IAMAuthEnabled" [plain @305:28..305:42] !<!Ref>
            "DBPort" [plain @306:7..306:13]: "Port" [plain @306:20..306:24] !<!Ref>
            "PreferredBackupWindow" [plain @307:7..307:28]: "NeptuneDBClusterPreferredBackupWindow" [plain @307:35..307:72] !<!Ref>
            "PreferredMaintenanceWindow" [plain @308:7..308:33]: "NeptuneDBClusterPreferredMaintenanceWindow" [plain @308:40..308:82] !<!Ref>
            "StorageEncrypted" [plain @311:7..311:23]: "StorageEncrypted" [plain @311:30..311:46] !<!Ref>
            "VpcSecurityGroupIds" [plain @315:7..315:26]: sequence [block]
              - "NeptuneDBSG" [plain @316:16..316:27] !<!Ref>
            "Tags" [plain @319:7..319:11]: sequence [block]
              - mapping [block]
                "Key" [plain @320:11..320:14]: "Name" [plain @320:16..320:20]
                "Value" [plain @321:11..321:16]: "${Env}-${AppName}-Cluster" [plain @321:23..321:48] !<!Sub>
              - mapping [block]
                "Key" [plain @322:11..322:14]: "App" [plain @322:16..322:19]
                "Value" [plain @323:11..323:16]: "${AppName}" [plain @323:23..323:33] !<!Sub>
              - mapping [block]
                "Key" [plain @324:11..324:14]: "Compliance" [plain @324:16..324:26]
                "Value" [plain @325:11..325:16]: "Compliance" [plain @325:18..325:28]
              - mapping [block]
                "Key" [plain @326:11..326:14]: "Env" [plain @326:16..326:19]
                "Value" [plain @327:11..327:16]: "${Env}" [plain @327:23..327:29] !<!Sub>
              - mapping [block]
                "Key" [plain @328:11..328:14]: "User" [plain @328:16..328:20]
                "Value" [plain @329:11..329:16]: "${User}" [plain @329:23..329:30] !<!Sub>
              - mapping [block]
                "Key" [plain @330:11..330:14]: "Owner" [plain @330:16..330:21]
                "Value" [plain @331:11..331:16]: "${Owner}" [plain @331:23..331:31] !<!Sub>
              - mapping [block]
                "Key" [plain @332:11..332:14]: "Tier" [plain @332:16..332:20]
                "Value" [plain @333:11..333:16]: "${Tier}" [plain @333:23..333:30] !<!Sub>
              - mapping [block]
                "Key" [plain @334:11..334:14]: "Version" [plain @334:16..334:23]
                "Value" [plain @335:11..335:16]: "${Version}" [plain @335:23..335:33] !<!Sub>
              - mapping [block]
                "Key" [plain @336:11..336:14]: "Storage" [plain @336:16..336:23]
                "Value" [plain @337:11..337:16]: "${Storage}" [plain @337:23..337:33] !<!Sub>
        "NeptuneDBInstance" [plain @340:3..340:20]: mapping [block]
          "Type" [plain @341:5..341:9]: "AWS::Neptune::DBInstance" [plain @341:11..341:35]
          "Properties" [plain @342:5..342:15]: mapping [block]
            "AllowMajorVersionUpgrade" [plain @343:7..343:31]: "MajorVersionUpgrade" [plain @343:38..343:57] !<!Ref>
            "AutoMinorVersionUpgrade" [plain @344:7..344:30]: "MinorVersionUpgrade" [plain @344:37..344:56] !<!Ref>
            "DBClusterIdentifier" [plain @345:7..345:26]: "NeptuneDBCluster" [plain @345:33..345:49] !<!Ref>
            "DBInstanceClass" [plain @346:7..346:22]: "DBInstanceClass" [plain @346:29..346:44] !<!Ref>
            "DBParameterGroupName" [plain @349:7..349:27]: "NeptuneDBParameterGroup" [plain @349:34..349:57] !<!Ref>
            "DBSubnetGroupName" [plain @352:7..352:24]: "NeptuneDBSubnetGroup" [plain @352:31..352:51] !<!Ref>
            "PreferredMaintenanceWindow" [plain @353:7..353:33]: "NeptuneDBInstancePreferredMaintenanceWindow" [plain @353:40..353:83] !<!Ref>
            "Tags" [plain @354:7..354:11]: sequence [block]
              - mapping [block]
                "Key" [plain @355:11..355:14]: "Name" [plain @355:16..355:20]
                "Value" [plain @356:11..356:16]: "${Env}-${AppName}-Instance" [plain @356:23..356:49] !<!Sub>
              - mapping [block]
                "Key" [plain @357:11..357:14]: "App" [plain @357:16..357:19]
                "Value" [plain @358:11..358:16]: "${AppName}" [plain @358:23..358:33] !<!Sub>
              - mapping [block]
                "Key" [plain @359:11..359:14]: "Compliance" [plain @359:16..359:26]
                "Value" [plain @360:11..360:16]: "Compliance" [plain @360:18..360:28]
              - mapping [block]
                "Key" [plain @361:11..361:14]: "Env" [plain @361:16..361:19]
                "Value" [plain @362:11..362:16]: "${Env}" [plain @362:23..362:29] !<!Sub>
              - mapping [block]
                "Key" [plain @363:11..363:14]: "User" [plain @363:16..363:20]
                "Value" [plain @364:11..364:16]: "${User}" [plain @364:23..364:30] !<!Sub>
              - mapping [block]
                "Key" [plain @365:11..365:14]: "Owner" [plain @365:16..365:21]
                "Value" [plain @366:11..366:16]: "${Owner}" [plain @366:23..366:31] !<!Sub>
              - mapping [block]
                "Key" [plain @367:11..367:14]: "Tier" [plain @367:16..367:20]
                "Value" [plain @368:11..368:16]: "${Tier}" [plain @368:23..368:30] !<!Sub>
              - mapping [block]
                "Key" [plain @369:11..369:14]: "Version" [plain @369:16..369:23]
                "Value" [plain @370:11..370:16]: "${Version}" [plain @370:23..370:33] !<!Sub>
              - mapping [block]
                "Key" [plain @371:11..371:14]: "Storage" [plain @371:16..371:23]
                "Value" [plain @372:11..372:16]: "${Storage}" [plain @372:23..372:33] !<!Sub>
        "NeptuneDBClusterParameterGroup" [plain @375:3..375:33]: mapping [block]
          "Type" [plain @376:5..376:9]: "AWS::Neptune::DBClusterParameterGroup" [plain @376:11..376:48]
          "Properties" [plain @377:5..377:15]: mapping [block]
            "Description" [plain @378:7..378:18]: "CloudFormation managed Neptune DB Cluster Parameter Group - ${Env}-${AppName}-cluster-parameter-group" [plain @378:25..378:126] !<!Sub>
            "Parameters" [plain @379:7..379:17]: mapping [block]
              "neptune_enable_audit_log" [plain @380:9..380:33]: sequence [block] !<!If>
                - "EnableAuditLogUpload" [plain @381:13..381:33]
                - "1" [plain @382:13..382:14]
                - "0" [plain @383:13..383:14]
            "Family" [plain @384:7..384:13]: "neptune1" [plain @384:15..384:23]
            "Name" [plain @385:7..385:11]: "${Env}-${AppName}-neptune-cluster-parameter-group" [plain @385:18..385:67] !<!Sub>
            "Tags" [plain @386:7..386:11]: sequence [block]
              - mapping [block]
                "Key" [plain @387:11..387:14]: "Name" [plain @387:16..387:20]
                "Value" [plain @388:11..388:16]: "${Env}-${AppName}-cluster-parameter-group" [plain @388:23..388:64] !<!Sub>
              - mapping [block]
                "Key" [plain @389:11..389:14]: "App" [plain @389:16..389:19]
                "Value" [plain @390:11..390:16]: "${AppName}" [plain @390:23..390:33] !<!Sub>
              - mapping [block]
                "Key" [plain @391:11..391:14]: "Compliance" [plain @391:16..391:26]
                "Value" [plain @392:11..392:16]: "Compliance" [plain @392:18..392:28]
              - mapping [block]
                "Key" [plain @393:11..393:14]: "Env" [plain @393:16..393:19]
                "Value" [plain @394:11..394:16]: "${Env}" [plain @394:23..394:29] !<!Sub>
              - mapping [block]
                "Key" [plain @395:11..395:14]: "User" [plain @395:16..395:20]
                "Value" [plain @396:11..396:16]: "${User}" [plain @396:23..396:30] !<!Sub>
              - mapping [block]
                "Key" [plain @397:11..397:14]: "Owner" [plain @397:16..397:21]
                "Value" [plain @398:11..398:16]: "${Owner}" [plain @398:23..398:31] !<!Sub>
              - mapping [block]
                "Key" [plain @399:11..399:14]: "Tier" [plain @399:16..399:20]
                "Value" [plain @400:11..400:16]: "${Tier}" [plain @400:23..400:30] !<!Sub>
              - mapping [block]
                "Key" [plain @401:11..401:14]: "Version" [plain @401:16..401:23]
                "Value" [plain @402:11..402:16]: "${Version}" [plain @402:23..402:33] !<!Sub>
              - mapping [block]
                "Key" [plain @403:11..403:14]: "Storage" [plain @403:16..403:23]
                "Value" [plain @404:11..404:16]: "${Storage}" [plain @404:23..404:33] !<!Sub>
        "NeptuneDBParameterGroup" [plain @407:3..407:26]: mapping [block]
          "Type" [plain @408:5..408:9]: "AWS::Neptune::DBParameterGroup" [plain @408:11..408:41]
          "Properties" [plain @409:5..409:15]: mapping [block]
            "Description" [plain @410:7..410:18]: "CloudFormation managed Neptune DB Parameter Group - ${Env}-${AppName}-parameter-group" [plain @410:25..410:110] !<!Sub>
            "Parameters" [plain @411:7..411:17]: mapping [block]
              "neptune_query_timeout" [plain @412:9..412:30]: "NeptuneQueryTimeout" [plain @412:37..412:56] !<!Ref>
            "Family" [plain @413:7..413:13]: "neptune1" [plain @413:15..413:23]
            "Name" [plain @414:7..414:11]: "${Env}-${AppName}-parameter-group" [plain @414:18..414:51] !<!Sub>
            "Tags" [plain @415:7..415:11]: sequence [block]
              - mapping [block]
                "Key" [plain @416:11..416:14]: "Name" [plain @416:16..416:20]
                "Value" [plain @417:11..417:16]: "${Env}-${AppName}-parameter-group" [plain @417:23..417:56] !<!Sub>
              - mapping [block]
                "Key" [plain @418:11..418:14]: "App" [plain @418:16..418:19]
                "Value" [plain @419:11..419:16]: "${AppName}" [plain @419:23..419:33] !<!Sub>
              - mapping [block]
                "Key" [plain @420:11..420:14]: "Compliance" [plain @420:16..420:26]
                "Value" [plain @421:11..421:16]: "Compliance" [plain @421:18..421:28]
              - mapping [block]
                "Key" [plain @422:11..422:14]: "Env" [plain @422:16..422:19]
                "Value" [plain @423:11..423:16]: "${Env}" [plain @423:23..423:29] !<!Sub>
              - mapping [block]
                "Key" [plain @424:11..424:14]: "User" [plain @424:16..424:20]
                "Value" [plain @425:11..425:16]: "${User}" [plain @425:23..425:30] !<!Sub>
              - mapping [block]
                "Key" [plain @426:11..426:14]: "Owner" [plain @426:16..426:21]
                "Value" [plain @427:11..427:16]: "${Owner}" [plain @427:23..427:31] !<!Sub>
              - mapping [block]
                "Key" [plain @428:11..428:14]: "Tier" [plain @428:16..428:20]
                "Value" [plain @429:11..429:16]: "${Tier}" [plain @429:23..429:30] !<!Sub>
              - mapping [block]
                "Key" [plain @430:11..430:14]: "Version" [plain @430:16..430:23]
                "Value" [plain @431:11..431:16]: "${Version}" [plain @431:23..431:33] !<!Sub>
              - mapping [block]
                "Key" [plain @432:11..432:14]: "Storage" [plain @432:16..432:23]
                "Value" [plain @433:11..433:16]: "${Storage}" [plain @433:23..433:33] !<!Sub>
        "NeptuneDBSubnetGroup" [plain @436:3..436:23]: mapping [block]
          "Type" [plain @437:5..437:9]: "AWS::Neptune::DBSubnetGroup" [plain @437:11..437:38]
          "Properties" [plain @438:5..438:15]: mapping [block]
            "DBSubnetGroupDescription" [plain @439:7..439:31]: "CloudFormation managed Neptune DB Subnet Group - ${Env}-${AppName}-subnet-group" [plain @439:38..439:117] !<!Sub>
            "DBSubnetGroupName" [plain @440:7..440:24]: "NeptuneDBSubnetGroupName" [plain @440:31..440:55] !<!Ref>
            "SubnetIds" [plain @441:7..441:16]: sequence [block]
              - mapping [block] !<!ImportValue>
                "Fn::Sub" [plain @443:11..443:18]: "${VPCStack}-PrivateSubnet1" [plain @443:20..443:46]
              - mapping [block] !<!ImportValue>
                "Fn::Sub" [plain @445:11..445:18]: "${VPCStack}-PrivateSubnet2" [plain @445:20..445:46]
            "Tags" [plain @446:7..446:11]: sequence [block]
              - mapping [block]
                "Key" [plain @447:11..447:14]: "Name" [plain @447:16..447:20]
                "Value" [plain @448:11..448:16]: "${Env}-${AppName}-subnet-group" [plain @448:23..448:53] !<!Sub>
              - mapping [block]
                "Key" [plain @449:11..449:14]: "App" [plain @449:16..449:19]
                "Value" [plain @450:11..450:16]: "${AppName}" [plain @450:23..450:33] !<!Sub>
              - mapping [block]
                "Key" [plain @451:11..451:14]: "Compliance" [plain @451:16..451:26]
                "Value" [plain @452:11..452:16]: "Compliance" [plain @452:18..452:28]
              - mapping [block]
                "Key" [plain @453:11..453:14]: "Env" [plain @453:16..453:19]
                "Value" [plain @454:11..454:16]: "${Env}" [plain @454:23..454:29] !<!Sub>
              - mapping [block]
                "Key" [plain @455:11..455:14]: "User" [plain @455:16..455:20]
                "Value" [plain @456:11..456:16]: "${User}" [plain @456:23..456:30] !<!Sub>
              - mapping [block]
                "Key" [plain @457:11..457:14]: "Owner" [plain @457:16..457:21]
                "Value" [plain @458:11..458:16]: "${Owner}" [plain @458:23..458:31] !<!Sub>
              - mapping [block]
                "Key" [plain @459:11..459:14]: "Tier" [plain @459:16..459:20]
                "Value" [plain @460:11..460:16]: "${Tier}" [plain @460:23..460:30] !<!Sub>
              - mapping [block]
                "Key" [plain @461:11..461:14]: "Version" [plain @461:16..461:23]
                "Value" [plain @462:11..462:16]: "${Version}" [plain @462:23..462:33] !<!Sub>
              - mapping [block]
                "Key" [plain @463:11..463:14]: "Storage" [plain @463:16..463:23]
                "Value" [plain @464:11..464:16]: "${Storage}" [plain @464:23..464:33] !<!Sub>
        "NeptuneAlarmTopic" [plain @468:3..468:20]: mapping [block]
          "Type" [plain @469:5..469:9]: "AWS::SNS::Topic" [plain @469:11..469:26]
          "Properties" [plain @470:5..470:15]: mapping [block]
            "DisplayName" [plain @471:7..471:18]: "${AWS::StackName} alarm topic" [plain @471:25..471:54] !<!Sub>
          "Condition" [plain @472:5..472:14]: "CreateSnsTopic" [plain @472:16..472:30]
        "NeptuneAlarmSubscription" [plain @475:3..475:27]: mapping [block]
          "Type" [plain @476:5..476:9]: "AWS::SNS::Subscription" [plain @476:11..476:33]
          "Properties" [plain @477:5..477:15]: mapping [block]
            "Endpoint" [plain @478:7..478:15]: "SNSEmailSubscription" [plain @478:22..478:42] !<!Ref>
            "Protocol" [plain @479:7..479:15]: "email" [plain @479:17..479:22]
            "TopicArn" [plain @480:7..480:15]: sequence [block] !<!If>
              - "CreateSnsTopic" [plain @481:11..481:25]
              - "NeptuneAlarmTopic" [plain @482:16..482:33] !<!Ref>
              - "NeptuneSNSTopicArn" [plain @483:16..483:34] !<!Ref>
          "Condition" [plain @486:5..486:14]: "CreateSnsSubscription" [plain @486:16..486:37]
        "NeptuneRole" [plain @489:3..489:14]: mapping [block]
          "Type" [plain @490:5..490:9]: "AWS::IAM::Role" [plain @490:11..490:25]
          "Properties" [plain @491:5..491:15]: mapping [block]
            "RoleName" [plain @492:7..492:15]: "${Env}-${AppName}-neptune-iam-role-${AWS::Region}" [plain @492:22..492:71] !<!Sub>
            "AssumeRolePolicyDocument" [plain @493:7..493:31]: mapping [block]
              "Version" [plain @494:9..494:16]: "2012-10-17" [double-quoted @494:18..494:30]
              "Statement" [plain @495:9..495:18]: sequence [block]
                - mapping [block]
                  "Effect" [plain @496:13..496:19]: "Allow" [plain @496:21..496:26]
                  "Principal" [plain @497:13..497:22]: mapping [block]
                    "Service" [plain @498:15..498:22]: sequence [block]
                      - "monitoring.rds.amazonaws.com" [plain @499:19..499:47]
                      - "rds.amazonaws.com" [plain @500:19..500:36]
                  "Action" [plain @501:13..501:19]: "sts:AssumeRole" [plain @501:21..501:35]
            "ManagedPolicyArns" [plain @502:7..502:24]: sequence [block]
              - "NeptuneCloudWatchPolicy" [plain @503:16..503:39] !<!Ref>
              - "NeptuneS3Policy" [plain @504:16..504:31] !<!Ref>
          "Condition" [plain @505:5..505:14]: "EnableAuditLogUpload" [plain @505:16..505:36]
        "NeptuneCloudWatchPolicy" [plain @508:3..508:26]: mapping [block]
          "Type" [plain @509:5..509:9]: "AWS::IAM::ManagedPolicy" [plain @509:11..509:34]
          "Properties" [plain @510:5..510:15]: mapping [block]
            "Description" [plain @511:7..511:18]: "Default policy for CloudWatch logs" [plain @511:20..511:54]
            "ManagedPolicyName" [plain @512:7..512:24]: "${Env}-${AppName}-neptune-cw-policy-${AWS::Region}" [plain @512:31..512:81] !<!Sub>
            "PolicyDocument" [plain @513:7..513:21]: mapping [block]
              "Version" [plain @514:9..514:16]: "2012-10-17" [double-quoted @514:18..514:30]
              "Statement" [plain @515:9..515:18]: sequence [block]
                - mapping [block]
                  "Sid" [plain @516:13..516:16]: "EnableLogGroups" [plain @516:18..516:33]
                  "Effect" [plain @517:13..517:19]: "Allow" [plain @517:21..517:26]
                  "Action" [plain @518:13..518:19]: sequence [block]
                    - "logs:CreateLogGroup" [plain @519:17..519:36]
                    - "logs:PutRetentionPolicy" [plain @520:17..520:40]
                  "Resource" [plain @521:13..521:21]: sequence [block]
                    - "arn:${AWS::Partition}:logs:*:*:log-group:/aws/neptune/*" [plain @522:22..522:77] !<!Sub>
                - mapping [block]
                  "Sid" [plain @523:13..523:16]: "EnableLogStreams" [plain @523:18..523:34]
                  "Effect" [plain @524:13..524:19]: "Allow" [plain @524:21..524:26]
                  "Action" [plain @525:13..525:19]: sequence [block]
                    - "logs:CreateLogStream" [plain @526:17..526:37]
                    - "logs:PutLogEvents" [plain @527:17..527:34]
                    - "logs:DescribeLogStreams" [plain @528:17..528:40]
                    - "logs:GetLogEvents" [plain @529:17..529:34]
                  "Resource" [plain @530:13..530:21]: sequence [block]
                    - "arn:${AWS::Partition}:logs:*:*:log-group:/aws/neptune/*:log-stream:*" [plain @531:22..531:90] !<!Sub>
        "NeptuneS3Policy" [plain @534:3..534:18]: mapping [block]
          "Type" [plain @535:5..535:9]: "AWS::IAM::ManagedPolicy" [plain @535:11..535:34]
          "Properties" [plain @536:5..536:15]: mapping [block]
            "Description" [plain @537:7..537:18]: "Neptune default policy for S3 access for data load" [plain @537:20..537:70]
            "ManagedPolicyName" [plain @538:7..538:24]: "${Env}-${AppName}-neptune-s3-policy-${AWS::Region}" [plain @538:31..538:81] !<!Sub>
            "PolicyDocument" [plain @539:7..539:21]: mapping [block]
              "Version" [plain @540:9..540:16]: "2012-10-17" [double-quoted @540:18..540:30]
              "Statement" [plain @541:9..541:18]: sequence [block]
                - mapping [block]
                  "Sid" [plain @542:13..542:16]: "AllowNeptuneAccessToS3" [plain @542:18..542:40]
                  "Effect" [plain @543:13..543:19]: "Allow" [plain @543:21..543:26]
                  "Action" [plain @544:13..544:19]: sequence [block]
                    - "s3:Get*" [plain @545:17..545:24]
                    - "s3:List*" [plain @546:17..546:25]
                  "Resource" [plain @547:13..547:21]: sequence [block]
                    - "arn:${AWS::Partition}:s3:::*" [plain @548:22..548:50] !<!Sub>
        "NeptunePrimaryCpuAlarm" [plain @552:3..552:25]: mapping [block]
          "Type" [plain @553:5..553:9]: "AWS::CloudWatch::Alarm" [plain @553:11..553:33]
          "Properties" [plain @554:5..554:15]: mapping [block]
            "AlarmDescription" [plain @555:7..555:23]: "${Env}-${AppName} primary DB CPU over ${HighCpuAlarmThreshold}%" [plain @555:30..555:93] !<!Sub>
            "Namespace" [plain @556:7..556:16]: "AWS/Neptune" [plain @556:18..556:29]
            "MetricName" [plain @557:7..557:17]: "CPUUtilization" [plain @557:19..557:33]
            "Unit" [plain @558:7..558:11]: "Percent" [plain @558:13..558:20]
            "Statistic" [plain @559:7..559:16]: "Average" [plain @559:18..559:25]
            "Period" [plain @560:7..560:13]: "300" [plain @560:15..560:18]
            "EvaluationPeriods" [plain @561:7..561:24]: "2" [plain @561:26..561:27]
            "Threshold" [plain @562:7..562:16]: "HighCpuAlarmThreshold" [plain @562:23..562:44] !<!Ref>
            "ComparisonOperator" [plain @563:7..563:25]: "GreaterThanOrEqualToThreshold" [plain @563:27..563:56]
            "Dimensions" [plain @564:7..564:17]: sequence [block]
              - mapping [block]
                "Name" [plain @565:11..565:15]: "DBClusterIdentifier" [plain @565:17..565:36]
                "Value" [plain @566:11..566:16]: "NeptuneDBCluster" [plain @566:23..566:39] !<!Ref>
            "AlarmActions" [plain @567:7..567:19]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @569:13..569:27]
                - "NeptuneAlarmTopic" [plain @570:18..570:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @571:18..571:36] !<!Ref>
            "InsufficientDataActions" [plain @572:7..572:30]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @574:13..574:27]
                - "NeptuneAlarmTopic" [plain @575:18..575:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @576:18..576:36] !<!Ref>
        "NeptunePrimaryMemoryAlarm" [plain @579:3..579:28]: mapping [block]
          "Type" [plain @580:5..580:9]: "AWS::CloudWatch::Alarm" [plain @580:11..580:33]
          "Properties" [plain @581:5..581:15]: mapping [block]
            "AlarmDescription" [plain @582:7..582:23]: "${Env}-${AppName} primary DB memory under ${LowMemoryAlarmThreshold} bytes" [plain @582:30..582:104] !<!Sub>
            "Namespace" [plain @583:7..583:16]: "AWS/Neptune" [plain @583:18..583:29]
            "MetricName" [plain @584:7..584:17]: "FreeableMemory" [plain @584:19..584:33]
            "Unit" [plain @585:7..585:11]: "Bytes" [plain @585:13..585:18]
            "Statistic" [plain @586:7..586:16]: "Average" [plain @586:18..586:25]
            "Period" [plain @587:7..587:13]: "300" [plain @587:15..587:18]
            "EvaluationPeriods" [plain @588:7..588:24]: "2" [plain @588:26..588:27]
            "Threshold" [plain @589:7..589:16]: "LowMemoryAlarmThreshold" [plain @589:23..589:46] !<!Ref>
            "ComparisonOperator" [plain @590:7..590:25]: "LessThanOrEqualToThreshold" [plain @590:27..590:53]
            "Dimensions" [plain @591:7..591:17]: sequence [block]
              - mapping [block]
                "Name" [plain @592:11..592:15]: "DBClusterIdentifier" [plain @592:17..592:36]
                "Value" [plain @593:11..593:16]: "NeptuneDBCluster" [plain @593:23..593:39] !<!Ref>
            "AlarmActions" [plain @594:7..594:19]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @596:13..596:27]
                - "NeptuneAlarmTopic" [plain @597:18..597:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @598:18..598:36] !<!Ref>
            "InsufficientDataActions" [plain @599:7..599:30]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @601:13..601:27]
                - "NeptuneAlarmTopic" [plain @602:18..602:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @603:18..603:36] !<!Ref>
        "NeptunePrimaryGremlinRequestsPerSecAlarm" [plain @606:3..606:43]: mapping [block]
          "Type" [plain @607:5..607:9]: "AWS::CloudWatch::Alarm" [plain @607:11..607:33]
          "Properties" [plain @608:5..608:15]: mapping [block]
            "AlarmDescription" [plain @609:7..609:23]: "${Env}-${AppName} primary DB Gremlin Requests Per Second" [plain @609:30..609:86] !<!Sub>
            "Namespace" [plain @610:7..610:16]: "AWS/Neptune" [plain @610:18..610:29]
            "MetricName" [plain @611:7..611:17]: "GremlinRequestsPerSec" [plain @611:19..611:40]
            "Statistic" [plain @612:7..612:16]: "Average" [plain @612:18..612:25]
            "Period" [plain @613:7..613:13]: "300" [plain @613:15..613:18]
            "EvaluationPeriods" [plain @614:7..614:24]: "2" [plain @614:26..614:27]
            "Threshold" [plain @615:7..615:16]: "GremlinRequestsPerSecThreshold" [plain @615:23..615:53] !<!Ref>
            "ComparisonOperator" [plain @616:7..616:25]: "GreaterThanOrEqualToThreshold" [plain @616:27..616:56]
            "Dimensions" [plain @617:7..617:17]: sequence [block]
              - mapping [block]
                "Name" [plain @618:11..618:15]: "DBClusterIdentifier" [plain @618:17..618:36]
                "Value" [plain @619:11..619:16]: "gremlin-cluster" [plain @619:18..619:33]
            "AlarmActions" [plain @620:7..620:19]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @622:13..622:27]
                - "NeptuneAlarmTopic" [plain @623:18..623:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @624:18..624:36] !<!Ref>
            "InsufficientDataActions" [plain @625:7..625:30]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @627:13..627:27]
                - "NeptuneAlarmTopic" [plain @628:18..628:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @629:18..629:36] !<!Ref>
        "NeptunePrimarySparqlRequestsPerSecAlarm" [plain @632:3..632:42]: mapping [block]
          "Type" [plain @633:5..633:9]: "AWS::CloudWatch::Alarm" [plain @633:11..633:33]
          "Properties" [plain @634:5..634:15]: mapping [block]
            "AlarmDescription" [plain @635:7..635:23]: "${Env}-${AppName} primary DB Sparql Requests Per Second" [plain @635:30..635:85] !<!Sub>
            "Namespace" [plain @636:7..636:16]: "AWS/Neptune" [plain @636:18..636:29]
            "MetricName" [plain @637:7..637:17]: "SparqlRequestsPerSec" [plain @637:19..637:39]
            "Statistic" [plain @638:7..638:16]: "Average" [plain @638:18..638:25]
            "Period" [plain @639:7..639:13]: "300" [plain @639:15..639:18]
            "EvaluationPeriods" [plain @640:7..640:24]: "2" [plain @640:26..640:27]
            "Threshold" [plain @641:7..641:16]: "SparqlRequestsPerSecThreshold" [plain @641:23..641:52] !<!Ref>
            "ComparisonOperator" [plain @642:7..642:25]: "GreaterThanOrEqualToThreshold" [plain @642:27..642:56]
            "Dimensions" [plain @643:7..643:17]: sequence [block]
              - mapping [block]
                "Name" [plain @644:11..644:15]: "DBClusterIdentifier" [plain @644:17..644:36]
                "Value" [plain @645:11..645:16]: "NeptuneDBCluster" [plain @645:23..645:39] !<!Ref>
            "AlarmActions" [plain @646:7..646:19]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @648:13..648:27]
                - "NeptuneAlarmTopic" [plain @649:18..649:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @650:18..650:36] !<!Ref>
            "InsufficientDataActions" [plain @651:7..651:30]: sequence [block]
              - sequence [block] !<!If>
                - "CreateSnsTopic" [plain @653:13..653:27]
                - "NeptuneAlarmTopic" [plain @654:18..654:35] !<!Ref>
                - "NeptuneSNSTopicArn" [plain @655:18..655:36] !<!Ref>
      "Outputs" [plain @657:1..657:8]: mapping [block]
        "NeptuneEndpointAddress" [plain @658:3..658:25]: mapping [block]
          "Description" [plain @659:5..659:16]: "Neptune DB endpoint address" [plain @659:18..659:45]
          "Value" [plain @660:5..660:10]: "NeptuneDBCluster.Endpoint" [plain @660:20..660:45] !<!GetAtt>
          "Export" [plain @661:5..661:11]: mapping [block]
            "Name" [plain @662:7..662:11]: "${Env}-${AppName}-neptune-endpoint-address" [plain @662:18..662:60] !<!Sub>
        "NeptuneReadEndpointAddress" [plain @664:3..664:29]: mapping [block]
          "Description" [plain @665:5..665:16]: "Neptune DB read-only endpoint address" [plain @665:18..665:55]
          "Value" [plain @666:5..666:10]: "NeptuneDBCluster.ReadEndpoint" [plain @666:20..666:49] !<!GetAtt>
          "Export" [plain @667:5..667:11]: mapping [block]
            "Name" [plain @668:7..668:11]: "${Env}-${AppName}-neptune-read-endpoint-address" [plain @668:18..668:65] !<!Sub>
        "NeptuneSnsTopic" [plain @670:3..670:18]: mapping [block]
          "Description" [plain @671:5..671:16]: "Neptune SNS Topic for alarms" [plain @671:18..671:46]
          "Value" [plain @672:5..672:10]: sequence [block] !<!If>
            - "CreateSnsTopic" [plain @673:9..673:23]
            - "NeptuneAlarmTopic" [plain @674:14..674:31] !<!Ref>
            - "NeptuneSNSTopicArn" [plain @675:14..675:32] !<!Ref>
          "Export" [plain @676:5..676:11]: mapping [block]
            "Name" [plain @677:7..677:11]: "${Env}-${AppName}-neptune-sns-topic" [plain @677:18..677:53] !<!Sub>
        "NeptuneEndpointPort" [plain @679:3..679:22]: mapping [block]
          "Description" [plain @680:5..680:16]: "Endpoint port" [plain @680:18..680:31]
          "Value" [plain @681:5..681:10]: "NeptuneDBCluster.Port" [plain @681:20..681:41] !<!GetAtt>
          "Export" [plain @682:5..682:11]: mapping [block]
            "Name" [plain @683:7..683:11]: "${Env}-${AppName}-neptune-endpoint-port" [plain @683:18..683:57] !<!Sub>
    "#);
}
