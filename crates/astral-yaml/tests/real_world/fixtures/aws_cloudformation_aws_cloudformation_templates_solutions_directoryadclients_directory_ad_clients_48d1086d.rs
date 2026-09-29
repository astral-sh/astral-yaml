#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-solutions-directoryadclients-directory-ad-clients-48d1086d"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "This template creates (1) Linux and (3) Windows EC2 instances and joins them to Active Directory using the 'AWS-JoinDirectoryServiceDomain' SSM document via AD Connector or AWS Managed AD directory. By default, it relies on the DNS servers being used by the EC2 instances knowing how to resolve the AD domain (i.e., Route 53 Resolvers, DHCP OptionsSet), with an option to set the DNS servers manually on the EC2 instances, as necessary. Several methods used to initiate the domain join (1) Windows EC2 instance with inline SSM association (2) Windows and Linux EC2 instance with SSM association targeting EC2 instance IDs (3) Windows EC2 instance with SSM association targeting EC2 instance tags" [plain @3:14..3:709]
      "Metadata" [plain @5:1..5:9]: mapping [block]
        "AWS::CloudFormation::Interface" [plain @6:3..6:33]: mapping [block]
          "ParameterGroups" [plain @7:5..7:20]: sequence [block]
            - mapping [block]
              "Label" [plain @8:9..8:14]: mapping [block]
                "default" [plain @9:11..9:18]: "Network Configuration" [plain @9:20..9:41]
              "Parameters" [plain @10:9..10:19]: sequence [block]
                - "PrivateSubnet1ID" [plain @11:13..11:29]
                - "PrivateSubnet2ID" [plain @12:13..12:29]
            - mapping [block]
              "Label" [plain @13:9..13:14]: mapping [block]
                "default" [plain @14:11..14:18]: "EC2 Instance Configuration" [plain @14:20..14:46]
              "Parameters" [plain @15:9..15:19]: sequence [block]
                - "DomainMembersInstanceType" [plain @16:13..16:38]
                - "DomainMember1NetBIOSName" [plain @17:13..17:37]
                - "DomainMember2NetBIOSName" [plain @18:13..18:37]
                - "DomainMember3NetBIOSName" [plain @19:13..19:37]
                - "DomainMember4NetBIOSName" [plain @20:13..20:37]
                - "KeyPairName" [plain @21:13..21:24]
                - "DomainMembersLinuxInstanceProfile" [plain @22:13..22:46]
                - "DomainMembersWindowsInstanceProfile" [plain @23:13..23:48]
                - "DomainMembersSGID" [plain @24:13..24:30]
                - "EBSKMSKey" [plain @25:13..25:22]
                - "AMAZONLINUX2" [plain @26:13..26:25]
                - "WINFULLBASE" [plain @27:13..27:24]
                - "SSMLogsBucketName" [plain @28:13..28:30]
            - mapping [block]
              "Label" [plain @29:9..29:14]: mapping [block]
                "default" [plain @30:11..30:18]: "Directory Services Configuration" [plain @30:20..30:52]
              "Parameters" [plain @31:9..31:19]: sequence [block]
                - "DirectoryID" [plain @32:13..32:24]
                - "DirectoryName" [plain @33:13..33:26]
                - "DomainDNSServer1" [plain @34:13..34:29]
                - "DomainDNSServer2" [plain @35:13..35:29]
                - "DomainDNSServer3" [plain @36:13..36:29]
                - "DomainDNSServer4" [plain @37:13..37:29]
          "ParameterLabels" [plain @38:5..38:20]: mapping [block]
            "AMAZONLINUX2" [plain @39:7..39:19]: mapping [block]
              "default" [plain @40:9..40:16]: "SSM Parameter Value for Lastest Amazon Linux 2 AMI ID" [plain @40:18..40:71]
            "DirectoryID" [plain @41:7..41:18]: mapping [block]
              "default" [plain @42:9..42:16]: "ID of the Directory (e.g., d-906764663a)" [plain @42:18..42:58]
            "DirectoryName" [plain @43:7..43:20]: mapping [block]
              "default" [plain @44:9..44:16]: "Directory Name" [plain @44:18..44:32]
            "DomainDNSServer1" [plain @45:7..45:23]: mapping [block]
              "default" [plain @46:9..46:16]: "Domain DNS Server 1" [plain @46:18..46:37]
            "DomainDNSServer2" [plain @47:7..47:23]: mapping [block]
              "default" [plain @48:9..48:16]: "Domain DNS Server 2" [plain @48:18..48:37]
            "DomainDNSServer3" [plain @49:7..49:23]: mapping [block]
              "default" [plain @50:9..50:16]: "Domain DNS Server 3" [plain @50:18..50:37]
            "DomainDNSServer4" [plain @51:7..51:23]: mapping [block]
              "default" [plain @52:9..52:16]: "Domain DNS Server 4" [plain @52:18..52:37]
            "DomainMember1NetBIOSName" [plain @53:7..53:31]: mapping [block]
              "default" [plain @54:9..54:16]: "Domain Member 1 NetBIOS Name" [plain @54:18..54:46]
            "DomainMember2NetBIOSName" [plain @55:7..55:31]: mapping [block]
              "default" [plain @56:9..56:16]: "Domain Member 2 NetBIOS Name" [plain @56:18..56:46]
            "DomainMember3NetBIOSName" [plain @57:7..57:31]: mapping [block]
              "default" [plain @58:9..58:16]: "Domain Member 3 NetBIOS Name" [plain @58:18..58:46]
            "DomainMember4NetBIOSName" [plain @59:7..59:31]: mapping [block]
              "default" [plain @60:9..60:16]: "Domain Member 4 NetBIOS Name" [plain @60:18..60:46]
            "DomainMembersInstanceType" [plain @61:7..61:32]: mapping [block]
              "default" [plain @62:9..62:16]: "Domain Members Instance Type" [plain @62:18..62:46]
            "DomainMembersSGID" [plain @63:7..63:24]: mapping [block]
              "default" [plain @64:9..64:16]: "Domain Members Security Group" [plain @64:18..64:47]
            "DomainMembersLinuxInstanceProfile" [plain @65:7..65:40]: mapping [block]
              "default" [plain @66:9..66:16]: "Domain Member Linux InstanceProfile" [plain @66:18..66:53]
            "DomainMembersWindowsInstanceProfile" [plain @67:7..67:42]: mapping [block]
              "default" [plain @68:9..68:16]: "Domain Member Windows InstanceProfile" [plain @68:18..68:55]
            "EBSKMSKey" [plain @69:7..69:16]: mapping [block]
              "default" [plain @70:9..70:16]: "Amazon EBS Volume KMS Key" [plain @70:18..70:43]
            "KeyPairName" [plain @71:7..71:18]: mapping [block]
              "default" [plain @72:9..72:16]: "Key Pair Name" [plain @72:18..72:31]
            "PrivateSubnet1ID" [plain @73:7..73:23]: mapping [block]
              "default" [plain @74:9..74:16]: "Private Subnet 1 ID" [plain @74:18..74:37]
            "PrivateSubnet2ID" [plain @75:7..75:23]: mapping [block]
              "default" [plain @76:9..76:16]: "Private Subnet 2 ID" [plain @76:18..76:37]
            "SSMLogsBucketName" [plain @77:7..77:24]: mapping [block]
              "default" [plain @78:9..78:16]: "SSM Logs Bucket Name" [plain @78:18..78:38]
            "WINFULLBASE" [plain @79:7..79:18]: mapping [block]
              "default" [plain @80:9..80:16]: "SSM Parameter Value for Lastest Windows AMI ID" [plain @80:18..80:64]
      "Parameters" [plain @82:1..82:11]: mapping [block]
        "AMAZONLINUX2" [plain @83:3..83:15]: mapping [block]
          "Type" [plain @84:5..84:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @84:11..84:58]
          "Default" [plain @85:5..85:12]: "/aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-ebs" [plain @85:14..85:75]
        "DirectoryID" [plain @87:3..87:14]: mapping [block]
          "Description" [plain @88:5..88:16]: "Directory ID" [plain @88:18..88:30]
          "Type" [plain @89:5..89:9]: "String" [plain @89:11..89:17]
          "AllowedPattern" [plain @90:5..90:19]: "^d-[0-9a-f]{10}$" [plain @90:21..90:37]
        "DirectoryName" [plain @92:3..92:16]: mapping [block]
          "Description" [plain @93:5..93:16]: "Fully qualified name of the on-premises directory, such as corp.example.com" [plain @93:18..93:93]
          "Type" [plain @94:5..94:9]: "String" [plain @94:11..94:17]
          "AllowedPattern" [plain @95:5..95:19]: "[a-zA-Z0-9-]+\\..+" [single-quoted @95:21..95:40]
          "MaxLength" [plain @96:5..96:14]: "25" [plain @96:16..96:18]
          "MinLength" [plain @97:5..97:14]: "3" [plain @97:16..97:17]
        "DomainDNSServer1" [plain @99:3..99:19]: mapping [block]
          "Description" [plain @100:5..100:16]: "(Optional) Domain DNS Server 1. If DNS servers are not set, then you need to ensure the DNS servers the EC2 instances are using can resolve the AD domain." [plain @100:18..100:172]
          "Type" [plain @101:5..101:9]: "String" [plain @101:11..101:17]
          "AllowedPattern" [plain @102:5..102:19]: "^$|^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$" [plain @102:21..102:129]
        "DomainDNSServer2" [plain @104:3..104:19]: mapping [block]
          "Description" [plain @105:5..105:16]: "(Optional) Domain DNS Server 2. If DNS servers are not set, then you need to ensure the DNS servers the EC2 instances are using can resolve the AD domain." [plain @105:18..105:172]
          "Type" [plain @106:5..106:9]: "String" [plain @106:11..106:17]
          "AllowedPattern" [plain @107:5..107:19]: "^$|^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$" [plain @107:21..107:129]
        "DomainDNSServer3" [plain @109:3..109:19]: mapping [block]
          "Description" [plain @110:5..110:16]: "(Optional) Domain DNS Server 3. If DNS servers are not set, then you need to ensure the DNS servers the EC2 instances are using can resolve the AD domain." [plain @110:18..110:172]
          "Type" [plain @111:5..111:9]: "String" [plain @111:11..111:17]
          "AllowedPattern" [plain @112:5..112:19]: "^$|^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$" [plain @112:21..112:129]
        "DomainDNSServer4" [plain @114:3..114:19]: mapping [block]
          "Description" [plain @115:5..115:16]: "(Optional) Domain DNS Server 4. If DNS servers are not set, then you need to ensure the DNS servers the EC2 instances are using can resolve the AD domain." [plain @115:18..115:172]
          "Type" [plain @116:5..116:9]: "String" [plain @116:11..116:17]
          "AllowedPattern" [plain @117:5..117:19]: "^$|^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$" [plain @117:21..117:129]
        "DomainMember1NetBIOSName" [plain @119:3..119:27]: mapping [block]
          "Description" [plain @120:5..120:16]: "NetBIOS name of Domain Member 1 (using inline SSM association). Note, if computer name existed in AD before, delete from AD first." [plain @120:18..120:148]
          "Type" [plain @121:5..121:9]: "String" [plain @121:11..121:17]
          "Default" [plain @122:5..122:12]: "SERVER1" [plain @122:14..122:21]
          "AllowedPattern" [plain @123:5..123:19]: "[a-zA-Z0-9-]+" [single-quoted @123:21..123:36]
          "MaxLength" [plain @124:5..124:14]: "15" [plain @124:16..124:18]
          "MinLength" [plain @125:5..125:14]: "1" [plain @125:16..125:17]
        "DomainMember2NetBIOSName" [plain @127:3..127:27]: mapping [block]
          "Description" [plain @128:5..128:16]: "NetBIOS name of Domain Member 2 (using SSM association via instance id). Note, if computer name existed in AD before, delete from AD first." [plain @128:18..128:157]
          "Type" [plain @129:5..129:9]: "String" [plain @129:11..129:17]
          "Default" [plain @130:5..130:12]: "SERVER2" [plain @130:14..130:21]
          "AllowedPattern" [plain @131:5..131:19]: "[a-zA-Z0-9-]+" [single-quoted @131:21..131:36]
          "MaxLength" [plain @132:5..132:14]: "15" [plain @132:16..132:18]
          "MinLength" [plain @133:5..133:14]: "1" [plain @133:16..133:17]
        "DomainMember3NetBIOSName" [plain @135:3..135:27]: mapping [block]
          "Description" [plain @136:5..136:16]: "NetBIOS name of Domain Member 3 (using SSM association via tag). Note, if computer name existed in AD before, delete from AD first." [plain @136:18..136:149]
          "Type" [plain @137:5..137:9]: "String" [plain @137:11..137:17]
          "Default" [plain @138:5..138:12]: "SERVER3" [plain @138:14..138:21]
          "AllowedPattern" [plain @139:5..139:19]: "[a-zA-Z0-9-]+" [single-quoted @139:21..139:36]
          "MaxLength" [plain @140:5..140:14]: "15" [plain @140:16..140:18]
          "MinLength" [plain @141:5..141:14]: "1" [plain @141:16..141:17]
        "DomainMember4NetBIOSName" [plain @143:3..143:27]: mapping [block]
          "Description" [plain @144:5..144:16]: "NetBIOS name of Domain Member 4 (AmazonLinux2)" [plain @144:18..144:64]
          "Type" [plain @145:5..145:9]: "String" [plain @145:11..145:17]
          "Default" [plain @146:5..146:12]: "SERVER4" [plain @146:14..146:21]
          "AllowedPattern" [plain @147:5..147:19]: "[a-zA-Z0-9-]+" [single-quoted @147:21..147:36]
          "MaxLength" [plain @148:5..148:14]: "15" [plain @148:16..148:18]
          "MinLength" [plain @149:5..149:14]: "1" [plain @149:16..149:17]
        "DomainMembersInstanceType" [plain @151:3..151:28]: mapping [block]
          "Description" [plain @152:5..152:16]: "Amazon EC2 instance type for the AD Server instances" [plain @152:18..152:70]
          "Type" [plain @153:5..153:9]: "String" [plain @153:11..153:17]
          "AllowedValues" [plain @154:5..154:18]: sequence [block]
            - "t3.medium" [plain @155:9..155:18]
            - "t3.large" [plain @156:9..156:17]
          "Default" [plain @157:5..157:12]: "t3.medium" [plain @157:14..157:23]
        "DomainMembersLinuxInstanceProfile" [plain @159:3..159:36]: mapping [block]
          "Description" [plain @160:5..160:16]: "Existing IAM InstanceProfile with Linux EC2 seamless join domain rights" [plain @160:18..160:89]
          "Type" [plain @161:5..161:9]: "String" [plain @161:11..161:17]
          "AllowedPattern" [plain @162:5..162:19]: "[\\w+=,.@-]+" [single-quoted @162:21..162:34]
        "DomainMembersWindowsInstanceProfile" [plain @164:3..164:38]: mapping [block]
          "Description" [plain @165:5..165:16]: "Existing IAM InstanceProfile with Windows EC2 seamless join domain rights" [plain @165:18..165:91]
          "Type" [plain @166:5..166:9]: "String" [plain @166:11..166:17]
          "AllowedPattern" [plain @167:5..167:19]: "[\\w+=,.@-]+" [single-quoted @167:21..167:34]
        "DomainMembersSGID" [plain @169:3..169:20]: mapping [block]
          "Description" [plain @170:5..170:16]: "Security Group ID for Domain Members Security Group" [plain @170:18..170:69]
          "Type" [plain @171:5..171:9]: "AWS::EC2::SecurityGroup::Id" [plain @171:11..171:38]
        "EBSKMSKey" [plain @173:3..173:12]: mapping [block]
          "Description" [plain @174:5..174:16]: "(Optional) KMS Alias, Key ID, Key ID ARN or Alias ARN to use for encrypting the EBS volumes. If empty, the default key for EBS encryption will be used of `alias/aws/ebs` or the CMK set as the default EBS encryption key." [plain @174:18..174:237]
          "Type" [plain @175:5..175:9]: "String" [plain @175:11..175:17]
          "AllowedPattern" [plain @176:5..176:19]: "^$|(^alias/[a-zA-Z0-9/-]{1,256}$)|(^[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}$)|(^arn:(aws[a-z-]*)?:kms:.*:\\d{12}:key/[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}$)|(^arn:(aws[a-z-]*)?:kms:.*:\\d{12}:alias/[a-zA-Z0-9/-]{1,256}$)" [plain @176:21..176:283]
        "KeyPairName" [plain @178:3..178:14]: mapping [block]
          "Description" [plain @179:5..179:16]: "KeyPair for ONPREMISES INSTANCES" [plain @179:18..179:50]
          "Type" [plain @180:5..180:9]: "AWS::EC2::KeyPair::KeyName" [plain @180:11..180:37]
        "PrivateSubnet1ID" [plain @182:3..182:19]: mapping [block]
          "Description" [plain @183:5..183:16]: "ID of the private subnet 1 in Availability Zone 1 (e.g., subnet-a0246dcd)" [plain @183:18..183:91]
          "Type" [plain @184:5..184:9]: "AWS::EC2::Subnet::Id" [plain @184:11..184:31]
        "PrivateSubnet2ID" [plain @186:3..186:19]: mapping [block]
          "Description" [plain @187:5..187:16]: "ID of the private subnet 2 in Availability Zone 2 (e.g., subnet-a0246dcd)" [plain @187:18..187:91]
          "Type" [plain @188:5..188:9]: "AWS::EC2::Subnet::Id" [plain @188:11..188:31]
        "SSMLogsBucketName" [plain @190:3..190:20]: mapping [block]
          "Description" [plain @191:5..191:16]: "(Optional) SSM Logs bucket name for where Systems Manager logs should store log files. SSM Logs bucket name can include numbers, lowercase letters, uppercase letters, and hyphens (-). It cannot start or end with a hyphen (-)." [plain @191:18..191:243]
          "Type" [plain @192:5..192:9]: "String" [plain @192:11..192:17]
          "AllowedPattern" [plain @193:5..193:19]: "^$|(?=^.{3,63}$)(?!.*[.-]{2})(?!.*[--]{2})(?!^(?:(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])(\\.(?!$)|$)){4}$)(^(([a-z0-9]|[a-z0-9][a-z0-9\\-]*[a-z0-9])\\.)*([a-z0-9]|[a-z0-9][a-z0-9\\-]*[a-z0-9])$)" [plain @193:21..193:215]
          "ConstraintDescription" [plain @194:5..194:26]: "SSM Logs bucket name can include numbers, lowercase letters, uppercase letters, and hyphens (-). It cannot start or end with a hyphen (-)." [plain @194:28..194:166]
        "WINFULLBASE" [plain @196:3..196:14]: mapping [block]
          "Type" [plain @197:5..197:9]: "AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>" [plain @197:11..197:58]
          "Default" [plain @198:5..198:12]: "/aws/service/ami-windows-latest/Windows_Server-2019-English-Full-Base" [plain @198:14..198:83]
      "Conditions" [plain @200:1..200:11]: mapping [block]
        "DomainDNSServer1Condition" [plain @201:3..201:28]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "DomainDNSServer1" [plain @203:14..203:30] !<!Ref>
            - "" [double-quoted @204:9..204:11]
        "DomainDNSServer2Condition" [plain @206:3..206:28]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "DomainDNSServer2" [plain @208:14..208:30] !<!Ref>
            - "" [double-quoted @209:9..209:11]
        "DomainDNSServer3Condition" [plain @211:3..211:28]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "DomainDNSServer3" [plain @213:14..213:30] !<!Ref>
            - "" [double-quoted @214:9..214:11]
        "DomainDNSServer4Condition" [plain @216:3..216:28]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "DomainDNSServer4" [plain @218:14..218:30] !<!Ref>
            - "" [double-quoted @219:9..219:11]
        "DomainDNSServersCondition" [plain @221:3..221:28]: sequence [block] !<!Or>
          - "DomainDNSServer1Condition" [plain @222:18..222:43] !<!Condition>
          - "DomainDNSServer2Condition" [plain @223:18..223:43] !<!Condition>
          - "DomainDNSServer3Condition" [plain @224:18..224:43] !<!Condition>
          - "DomainDNSServer4Condition" [plain @225:18..225:43] !<!Condition>
        "EBSKMSKeyCondition" [plain @227:3..227:21]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "EBSKMSKey" [plain @229:14..229:23] !<!Ref>
            - "" [double-quoted @230:9..230:11]
        "SSMLogsBucketCondition" [plain @232:3..232:25]: sequence [block] !<!Not>
          - sequence [block] !<!Equals>
            - "SSMLogsBucketName" [plain @234:14..234:31] !<!Ref>
            - "" [double-quoted @235:9..235:11]
      "Resources" [plain @237:1..237:10]: mapping [block]
        "DomainMember1WithInlineSsmAssociation" [plain @238:3..238:40]: mapping [block]
          "Type" [plain @239:5..239:9]: "AWS::EC2::Instance" [plain @239:11..239:29]
          "Properties" [plain @240:5..240:15]: mapping [block]
            "ImageId" [plain @241:7..241:14]: "WINFULLBASE" [plain @241:21..241:32] !<!Ref>
            "IamInstanceProfile" [plain @242:7..242:25]: "DomainMembersWindowsInstanceProfile" [plain @242:32..242:67] !<!Ref>
            "SsmAssociations" [plain @243:7..243:22]: sequence [block]
              - mapping [block]
                "DocumentName" [plain @244:11..244:23]: "AWS-JoinDirectoryServiceDomain" [plain @244:25..244:55]
                "AssociationParameters" [plain @245:11..245:32]: sequence [block]
                  - mapping [block]
                    "Key" [plain @246:15..246:18]: "directoryId" [plain @246:20..246:31]
                    "Value" [plain @247:15..247:20]: sequence [block]
                      - "DirectoryID" [plain @248:24..248:35] !<!Ref>
                  - mapping [block]
                    "Key" [plain @249:15..249:18]: "directoryName" [plain @249:20..249:33]
                    "Value" [plain @250:15..250:20]: sequence [block]
                      - "DirectoryName" [plain @251:24..251:37] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServersCondition" [plain @253:17..253:42]
                    - mapping [block]
                      "Key" [plain @254:17..254:20]: "dnsIpAddresses" [plain @254:22..254:36]
                      "Value" [plain @255:17..255:22]: sequence [block]
                        - sequence [block] !<!If>
                          - "DomainDNSServer1Condition" [plain @257:23..257:48]
                          - "DomainDNSServer1" [plain @258:28..258:44] !<!Ref>
                          - "AWS::NoValue" [plain @259:28..259:40] !<!Ref>
                        - sequence [block] !<!If>
                          - "DomainDNSServer2Condition" [plain @261:23..261:48]
                          - "DomainDNSServer2" [plain @262:28..262:44] !<!Ref>
                          - "AWS::NoValue" [plain @263:28..263:40] !<!Ref>
                        - sequence [block] !<!If>
                          - "DomainDNSServer3Condition" [plain @265:23..265:48]
                          - "DomainDNSServer3" [plain @266:28..266:44] !<!Ref>
                          - "AWS::NoValue" [plain @267:28..267:40] !<!Ref>
                        - sequence [block] !<!If>
                          - "DomainDNSServer4Condition" [plain @269:23..269:48]
                          - "DomainDNSServer4" [plain @270:28..270:44] !<!Ref>
                          - "AWS::NoValue" [plain @271:28..271:40] !<!Ref>
                    - "AWS::NoValue" [plain @272:22..272:34] !<!Ref>
            "InstanceType" [plain @273:7..273:19]: "DomainMembersInstanceType" [plain @273:26..273:51] !<!Ref>
            "SubnetId" [plain @274:7..274:15]: "PrivateSubnet1ID" [plain @274:22..274:38] !<!Ref>
            "Tags" [plain @275:7..275:11]: sequence [block]
              - mapping [block]
                "Key" [plain @276:11..276:14]: "Name" [plain @276:16..276:20]
                "Value" [plain @277:11..277:16]: "DomainMember1NetBIOSName" [plain @277:23..277:47] !<!Ref>
            "BlockDeviceMappings" [plain @278:7..278:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @279:11..279:21]: "/dev/sda1" [plain @279:23..279:32]
                "Ebs" [plain @280:11..280:14]: mapping [block]
                  "Encrypted" [plain @281:13..281:22]: "true" [plain @281:24..281:28]
                  "VolumeType" [plain @282:13..282:23]: "gp3" [plain @282:25..282:28]
                  "DeleteOnTermination" [plain @283:13..283:32]: "true" [plain @283:34..283:38]
                  "VolumeSize" [plain @284:13..284:23]: "100" [plain @284:25..284:28]
                  "KmsKeyId" [plain @285:13..285:21]: sequence [block] !<!If>
                    - "EBSKMSKeyCondition" [plain @286:17..286:35]
                    - "EBSKMSKey" [plain @287:22..287:31] !<!Ref>
                    - "AWS::NoValue" [plain @288:22..288:34] !<!Ref>
            "SecurityGroupIds" [plain @289:7..289:23]: sequence [block]
              - "DomainMembersSGID" [plain @290:16..290:33] !<!Ref>
            "KeyName" [plain @291:7..291:14]: "KeyPairName" [plain @291:21..291:32] !<!Ref>
            "UserData" [plain @292:7..292:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @293:9..293:16]: literal [@293:18..307:1]
                "<powershell>\n"
                "$instanceId = \"null\"\n"
                "while ($instanceId -NotLike \"i-*\") {\n"
                "Start-Sleep -s 3\n"
                "$instanceId = Invoke-RestMethod -uri http://169.254.169.254/latest/meta-data/instance-id\n"
                "}\n"
                "Rename-Computer -NewName ${DomainMember1NetBIOSName} -Force\n"
                "# Set-TimeZone -Name \"US Eastern Standard Time\"\n"
                "\n"
                "Install-WindowsFeature -IncludeAllSubFeature RSAT\n"
                "Restart-Computer -Force\n"
                "</powershell>\n"
        "DomainMember2WithSsmAssociationInstance" [plain @307:3..307:42]: mapping [block]
          "Type" [plain @308:5..308:9]: "AWS::EC2::Instance" [plain @308:11..308:29]
          "Properties" [plain @309:5..309:15]: mapping [block]
            "ImageId" [plain @310:7..310:14]: "WINFULLBASE" [plain @310:21..310:32] !<!Ref>
            "IamInstanceProfile" [plain @311:7..311:25]: "DomainMembersWindowsInstanceProfile" [plain @311:32..311:67] !<!Ref>
            "InstanceType" [plain @312:7..312:19]: "DomainMembersInstanceType" [plain @312:26..312:51] !<!Ref>
            "SubnetId" [plain @313:7..313:15]: "PrivateSubnet2ID" [plain @313:22..313:38] !<!Ref>
            "Tags" [plain @314:7..314:11]: sequence [block]
              - mapping [block]
                "Key" [plain @315:11..315:14]: "Name" [plain @315:16..315:20]
                "Value" [plain @316:11..316:16]: "DomainMember2NetBIOSName" [plain @316:23..316:47] !<!Ref>
            "BlockDeviceMappings" [plain @317:7..317:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @318:11..318:21]: "/dev/sda1" [plain @318:23..318:32]
                "Ebs" [plain @319:11..319:14]: mapping [block]
                  "Encrypted" [plain @320:13..320:22]: "true" [plain @320:24..320:28]
                  "VolumeType" [plain @321:13..321:23]: "gp3" [plain @321:25..321:28]
                  "DeleteOnTermination" [plain @322:13..322:32]: "true" [plain @322:34..322:38]
                  "VolumeSize" [plain @323:13..323:23]: "100" [plain @323:25..323:28]
                  "KmsKeyId" [plain @324:13..324:21]: sequence [block] !<!If>
                    - "EBSKMSKeyCondition" [plain @325:17..325:35]
                    - "EBSKMSKey" [plain @326:22..326:31] !<!Ref>
                    - "AWS::NoValue" [plain @327:22..327:34] !<!Ref>
            "SecurityGroupIds" [plain @328:7..328:23]: sequence [block]
              - "DomainMembersSGID" [plain @329:16..329:33] !<!Ref>
            "KeyName" [plain @330:7..330:14]: "KeyPairName" [plain @330:21..330:32] !<!Ref>
            "UserData" [plain @331:7..331:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @332:9..332:16]: literal [@332:18..346:1]
                "<powershell>\n"
                "$instanceId = \"null\"\n"
                "while ($instanceId -NotLike \"i-*\") {\n"
                "Start-Sleep -s 3\n"
                "$instanceId = Invoke-RestMethod -uri http://169.254.169.254/latest/meta-data/instance-id\n"
                "}\n"
                "Rename-Computer -NewName ${DomainMember2NetBIOSName} -Force\n"
                "# Set-TimeZone -Name \"US Eastern Standard Time\"\n"
                "\n"
                "Install-WindowsFeature -IncludeAllSubFeature RSAT\n"
                "Restart-Computer -Force\n"
                "</powershell>\n"
        "DomainMember3WithSsmAssociationTag" [plain @346:3..346:37]: mapping [block]
          "Type" [plain @347:5..347:9]: "AWS::EC2::Instance" [plain @347:11..347:29]
          "Properties" [plain @348:5..348:15]: mapping [block]
            "ImageId" [plain @349:7..349:14]: "WINFULLBASE" [plain @349:21..349:32] !<!Ref>
            "IamInstanceProfile" [plain @350:7..350:25]: "DomainMembersWindowsInstanceProfile" [plain @350:32..350:67] !<!Ref>
            "InstanceType" [plain @351:7..351:19]: "DomainMembersInstanceType" [plain @351:26..351:51] !<!Ref>
            "SubnetId" [plain @352:7..352:15]: "PrivateSubnet1ID" [plain @352:22..352:38] !<!Ref>
            "Tags" [plain @353:7..353:11]: sequence [block]
              - mapping [block]
                "Key" [plain @354:11..354:14]: "Name" [plain @354:16..354:20]
                "Value" [plain @355:11..355:16]: "DomainMember3NetBIOSName" [plain @355:23..355:47] !<!Ref>
              - mapping [block]
                "Key" [plain @356:11..356:14]: "DomainJoin" [plain @356:16..356:26]
                "Value" [plain @357:11..357:16]: "DirectoryName" [plain @357:23..357:36] !<!Ref>
            "BlockDeviceMappings" [plain @358:7..358:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @359:11..359:21]: "/dev/sda1" [plain @359:23..359:32]
                "Ebs" [plain @360:11..360:14]: mapping [block]
                  "Encrypted" [plain @361:13..361:22]: "true" [plain @361:24..361:28]
                  "VolumeType" [plain @362:13..362:23]: "gp3" [plain @362:25..362:28]
                  "DeleteOnTermination" [plain @363:13..363:32]: "true" [plain @363:34..363:38]
                  "VolumeSize" [plain @364:13..364:23]: "100" [plain @364:25..364:28]
                  "KmsKeyId" [plain @365:13..365:21]: sequence [block] !<!If>
                    - "EBSKMSKeyCondition" [plain @366:17..366:35]
                    - "EBSKMSKey" [plain @367:22..367:31] !<!Ref>
                    - "AWS::NoValue" [plain @368:22..368:34] !<!Ref>
            "SecurityGroupIds" [plain @369:7..369:23]: sequence [block]
              - "DomainMembersSGID" [plain @370:16..370:33] !<!Ref>
            "KeyName" [plain @371:7..371:14]: "KeyPairName" [plain @371:21..371:32] !<!Ref>
            "UserData" [plain @372:7..372:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @373:9..373:16]: literal [@373:18..387:1]
                "<powershell>\n"
                "$instanceId = \"null\"\n"
                "while ($instanceId -NotLike \"i-*\") {\n"
                "Start-Sleep -s 3\n"
                "$instanceId = Invoke-RestMethod -uri http://169.254.169.254/latest/meta-data/instance-id\n"
                "}\n"
                "Rename-Computer -NewName ${DomainMember3NetBIOSName} -Force\n"
                "# Set-TimeZone -Name \"US Eastern Standard Time\"\n"
                "\n"
                "Install-WindowsFeature -IncludeAllSubFeature RSAT\n"
                "Restart-Computer -Force\n"
                "</powershell>\n"
        "DomainMember4LinuxWithSsmAssociationInstance" [plain @387:3..387:47]: mapping [block]
          "Type" [plain @388:5..388:9]: "AWS::EC2::Instance" [plain @388:11..388:29]
          "Properties" [plain @389:5..389:15]: mapping [block]
            "ImageId" [plain @390:7..390:14]: "AMAZONLINUX2" [plain @390:21..390:33] !<!Ref>
            "IamInstanceProfile" [plain @391:7..391:25]: "DomainMembersLinuxInstanceProfile" [plain @391:32..391:65] !<!Ref>
            "InstanceType" [plain @392:7..392:19]: "DomainMembersInstanceType" [plain @392:26..392:51] !<!Ref>
            "SubnetId" [plain @393:7..393:15]: "PrivateSubnet2ID" [plain @393:22..393:38] !<!Ref>
            "Tags" [plain @394:7..394:11]: sequence [block]
              - mapping [block]
                "Key" [plain @395:11..395:14]: "Name" [plain @395:16..395:20]
                "Value" [plain @396:11..396:16]: "DomainMember4NetBIOSName" [plain @396:23..396:47] !<!Ref>
            "BlockDeviceMappings" [plain @397:7..397:26]: sequence [block]
              - mapping [block]
                "DeviceName" [plain @398:11..398:21]: "/dev/sda1" [plain @398:23..398:32]
                "Ebs" [plain @399:11..399:14]: mapping [block]
                  "Encrypted" [plain @400:13..400:22]: "true" [plain @400:24..400:28]
                  "VolumeType" [plain @401:13..401:23]: "gp3" [plain @401:25..401:28]
                  "DeleteOnTermination" [plain @402:13..402:32]: "true" [plain @402:34..402:38]
                  "VolumeSize" [plain @403:13..403:23]: "100" [plain @403:25..403:28]
                  "KmsKeyId" [plain @404:13..404:21]: sequence [block] !<!If>
                    - "EBSKMSKeyCondition" [plain @405:17..405:35]
                    - "EBSKMSKey" [plain @406:22..406:31] !<!Ref>
                    - "AWS::NoValue" [plain @407:22..407:34] !<!Ref>
            "SecurityGroupIds" [plain @408:7..408:23]: sequence [block]
              - "DomainMembersSGID" [plain @409:16..409:33] !<!Ref>
            "KeyName" [plain @410:7..410:14]: "KeyPairName" [plain @410:21..410:32] !<!Ref>
            "UserData" [plain @411:7..411:15]: mapping [block] !<!Base64>
              "Fn::Sub" [plain @412:9..412:16]: literal [@412:18..424:1]
                "# Set HostName\n"
                "LowerEc2Name=$(echo ${DomainMember4NetBIOSName} | tr '[:upper:]' '[:lower:]')\n"
                "hostnamectl set-hostname $LowerEc2Name\n"
                "# Set TimeZone\n"
                "# sed -i 's|^ZONE=.*|ZONE=\"America/New_York\"|' /etc/sysconfig/clock\n"
                "# ln -sf /usr/share/zoneinfo/America/New_York /etc/localtime\n"
                "# Patch System Up\n"
                "yum update -y\n"
                "# Reboot\n"
                "reboot\n"
        "JoinDomainAssociationInstances" [plain @424:3..424:33]: mapping [block]
          "Type" [plain @425:5..425:9]: "AWS::SSM::Association" [plain @425:11..425:32]
          "Properties" [plain @426:5..426:15]: mapping [block]
            "AssociationName" [plain @427:7..427:22]: "JoinDomain-Association-viaInstances-${AWS::StackName}" [plain @427:29..427:82] !<!Sub>
            "Name" [plain @428:7..428:11]: "AWS-JoinDirectoryServiceDomain" [plain @428:13..428:43]
            "OutputLocation" [plain @429:7..429:21]: mapping [block]
              "S3Location" [plain @430:9..430:19]: sequence [block] !<!If>
                - "SSMLogsBucketCondition" [plain @431:13..431:35]
                - mapping [block]
                  "OutputS3BucketName" [plain @432:13..432:31]: "SSMLogsBucketName" [plain @432:38..432:55] !<!Ref>
                  "OutputS3KeyPrefix" [plain @433:13..433:30]: "ssm-association-logs/AWSLogs/${AWS::AccountId}/*" [plain @433:37..433:85] !<!Sub>
                - "AWS::NoValue" [plain @434:18..434:30] !<!Ref>
            "Targets" [plain @435:7..435:14]: sequence [block]
              - mapping [block]
                "Key" [plain @436:11..436:14]: "InstanceIds" [plain @436:16..436:27]
                "Values" [plain @437:11..437:17]: sequence [block]
                  - "DomainMember2WithSsmAssociationInstance" [plain @438:20..438:59] !<!Ref>
                  - "DomainMember4LinuxWithSsmAssociationInstance" [plain @439:20..439:64] !<!Ref>
            "Parameters" [plain @440:7..440:17]: mapping [block]
              "directoryId" [plain @441:9..441:20]: sequence [block]
                - "DirectoryID" [plain @442:18..442:29] !<!Ref>
              "directoryName" [plain @443:9..443:22]: sequence [block]
                - "DirectoryName" [plain @444:18..444:31] !<!Ref>
              "dnsIpAddresses" [plain @445:9..445:23]: sequence [block] !<!If>
                - "DomainDNSServersCondition" [plain @446:13..446:38]
                - sequence [block]
                  - sequence [block] !<!If>
                    - "DomainDNSServer1Condition" [plain @448:17..448:42]
                    - "DomainDNSServer1" [plain @449:22..449:38] !<!Ref>
                    - "AWS::NoValue" [plain @450:22..450:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer2Condition" [plain @452:17..452:42]
                    - "DomainDNSServer2" [plain @453:22..453:38] !<!Ref>
                    - "AWS::NoValue" [plain @454:22..454:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer3Condition" [plain @456:17..456:42]
                    - "DomainDNSServer3" [plain @457:22..457:38] !<!Ref>
                    - "AWS::NoValue" [plain @458:22..458:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer4Condition" [plain @460:17..460:42]
                    - "DomainDNSServer4" [plain @461:22..461:38] !<!Ref>
                    - "AWS::NoValue" [plain @462:22..462:34] !<!Ref>
                - "AWS::NoValue" [plain @463:18..463:30] !<!Ref>
        "JoinDomainAssociationTags" [plain @465:3..465:28]: mapping [block]
          "Type" [plain @466:5..466:9]: "AWS::SSM::Association" [plain @466:11..466:32]
          "Properties" [plain @467:5..467:15]: mapping [block]
            "AssociationName" [plain @468:7..468:22]: "JoinDomain-Association-viaTags-${AWS::StackName}" [plain @468:29..468:77] !<!Sub>
            "Name" [plain @469:7..469:11]: "AWS-JoinDirectoryServiceDomain" [plain @469:13..469:43]
            "OutputLocation" [plain @470:7..470:21]: mapping [block]
              "S3Location" [plain @471:9..471:19]: sequence [block] !<!If>
                - "SSMLogsBucketCondition" [plain @472:13..472:35]
                - mapping [block]
                  "OutputS3BucketName" [plain @473:13..473:31]: "SSMLogsBucketName" [plain @473:38..473:55] !<!Ref>
                  "OutputS3KeyPrefix" [plain @474:13..474:30]: "ssm-association-logs/AWSLogs/${AWS::AccountId}/*" [plain @474:37..474:85] !<!Sub>
                - "AWS::NoValue" [plain @475:18..475:30] !<!Ref>
            "Targets" [plain @476:7..476:14]: sequence [block]
              - mapping [block]
                "Key" [plain @477:11..477:14]: "tag:DomainJoin" [plain @477:16..477:30]
                "Values" [plain @478:11..478:17]: sequence [block]
                  - "DirectoryName" [plain @479:20..479:33] !<!Ref>
            "Parameters" [plain @480:7..480:17]: mapping [block]
              "directoryId" [plain @481:9..481:20]: sequence [block]
                - "DirectoryID" [plain @482:18..482:29] !<!Ref>
              "directoryName" [plain @483:9..483:22]: sequence [block]
                - "DirectoryName" [plain @484:18..484:31] !<!Ref>
              "dnsIpAddresses" [plain @485:9..485:23]: sequence [block] !<!If>
                - "DomainDNSServersCondition" [plain @486:13..486:38]
                - sequence [block]
                  - sequence [block] !<!If>
                    - "DomainDNSServer1Condition" [plain @488:17..488:42]
                    - "DomainDNSServer1" [plain @489:22..489:38] !<!Ref>
                    - "AWS::NoValue" [plain @490:22..490:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer2Condition" [plain @492:17..492:42]
                    - "DomainDNSServer2" [plain @493:22..493:38] !<!Ref>
                    - "AWS::NoValue" [plain @494:22..494:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer3Condition" [plain @496:17..496:42]
                    - "DomainDNSServer3" [plain @497:22..497:38] !<!Ref>
                    - "AWS::NoValue" [plain @498:22..498:34] !<!Ref>
                  - sequence [block] !<!If>
                    - "DomainDNSServer4Condition" [plain @500:17..500:42]
                    - "DomainDNSServer4" [plain @501:22..501:38] !<!Ref>
                    - "AWS::NoValue" [plain @502:22..502:34] !<!Ref>
                - "AWS::NoValue" [plain @503:18..503:30] !<!Ref>
    "##);
}
