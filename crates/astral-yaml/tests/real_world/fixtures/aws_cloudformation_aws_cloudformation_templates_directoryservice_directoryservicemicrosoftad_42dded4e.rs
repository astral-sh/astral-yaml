#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-directoryservice-directoryservicemicrosoftad-42dded4e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: literal [@3:14..6:1]
        "Provision AWS Managed Active Directory\n"
      "Metadata" [plain @6:1..6:9]: mapping [block]
        "AWS::CloudFormation::Interface" [plain @7:3..7:33]: mapping [block]
          "ParameterGroups" [plain @8:5..8:20]: sequence [block]
            - mapping [block]
              "Label" [plain @9:9..9:14]: mapping [block]
                "default" [plain @10:11..10:18]: "Active Directory Settings" [plain @10:20..10:45]
              "Parameters" [plain @11:9..11:19]: sequence [block]
                - "pDomainName" [plain @12:13..12:24]
                - "pMicrosoftADShortName" [plain @13:13..13:34]
                - "pEdition" [plain @14:13..14:21]
                - "pCreateAlias" [plain @15:13..15:25]
                - "pEnableSingleSignOn" [plain @16:13..16:32]
            - mapping [block]
              "Label" [plain @17:9..17:14]: mapping [block]
                "default" [plain @18:11..18:18]: "VPC Networking Settings" [plain @18:20..18:43]
              "Parameters" [plain @19:9..19:19]: sequence [block]
                - "pVPCID" [plain @20:13..20:19]
                - "pPrivateSubnet1" [plain @21:13..21:28]
                - "pPrivateSubnet2" [plain @22:13..22:28]
          "ParameterLabels" [plain @23:5..23:20]: mapping [block]
            "pDomainName" [plain @24:7..24:18]: mapping [block]
              "default" [plain @25:9..25:16]: "Name" [plain @25:18..25:22]
            "pMicrosoftADShortName" [plain @26:7..26:28]: mapping [block]
              "default" [plain @27:9..27:16]: "Short Name" [plain @27:18..27:28]
            "pEdition" [plain @28:7..28:15]: mapping [block]
              "default" [plain @29:9..29:16]: "Edition" [plain @29:18..29:25]
            "pCreateAlias" [plain @30:7..30:19]: mapping [block]
              "default" [plain @31:9..31:16]: "Create Alias" [plain @31:18..31:30]
            "pEnableSingleSignOn" [plain @32:7..32:26]: mapping [block]
              "default" [plain @33:9..33:16]: "Enable Single Sign On" [plain @33:18..33:39]
            "pVPCID" [plain @34:7..34:13]: mapping [block]
              "default" [plain @35:9..35:16]: "VPC ID" [plain @35:18..35:24]
            "pPrivateSubnet1" [plain @36:7..36:22]: mapping [block]
              "default" [plain @37:9..37:16]: "Private Subnet 1" [plain @37:18..37:34]
            "pPrivateSubnet2" [plain @38:7..38:22]: mapping [block]
              "default" [plain @39:9..39:16]: "Private Subnet 2" [plain @39:18..39:34]
      "Parameters" [plain @41:1..41:11]: mapping [block]
        "pEdition" [plain @42:3..42:11]: mapping [block]
          "Description" [plain @43:5..43:16]: literal [@43:18..45:1]
            "The AWS Microsoft AD edition. Valid values include Standard and Enterprise. The default is Enterprise.\n"
          "Type" [plain @45:5..45:9]: "String" [plain @45:11..45:17]
          "AllowedValues" [plain @46:5..46:18]: sequence [block]
            - "Standard" [plain @47:9..47:17]
            - "Enterprise" [plain @48:9..48:19]
          "Default" [plain @49:5..49:12]: "Enterprise" [plain @49:14..49:24]
        "pDomainName" [plain @51:3..51:14]: mapping [block]
          "Description" [plain @52:5..52:16]: literal [@52:18..54:1]
            "The fully qualified name for the Microsoft Active Directory in AWS, such as corp.example.com. The name doesn't need to be publicly resolvable; it will resolve inside your VPC only.\n"
          "Type" [plain @54:5..54:9]: "String" [plain @54:11..54:17]
          "Default" [plain @55:5..55:12]: "corp.example.com" [plain @55:14..55:30]
        "pMicrosoftADShortName" [plain @57:3..57:24]: mapping [block]
          "Description" [plain @58:5..58:16]: literal [@58:18..60:1]
            "The NetBIOS name for your domain, such as CORP. If you don't specify a value, AWS Directory Service uses the first part of your directory DNS server name. For example, if your directory DNS server name is corp.example.com, AWS Directory Service specifies CORP for the NetBIOS name.\n"
          "Type" [plain @60:5..60:9]: "String" [plain @60:11..60:17]
          "Default" [plain @61:5..61:12]: "corp" [plain @61:14..61:18]
        "pEnableSingleSignOn" [plain @63:3..63:22]: mapping [block]
          "Description" [plain @64:5..64:16]: literal [@64:18..66:1]
            "Whether to enable single sign-on for a Microsoft Active Directory in AWS. Single sign-on allows users in your directory to access certain AWS services from a computer joined to the directory without having to enter their credentials separately. If you don't specify a value, AWS CloudFormation disables single sign-on by default. If enabling SSO, then \"Create Alias\" need to be set to true.\n"
          "Type" [plain @66:5..66:9]: "String" [plain @66:11..66:17]
          "AllowedValues" [plain @67:5..67:18]: sequence [block]
            - "true" [double-quoted @68:9..68:15]
            - "false" [double-quoted @69:9..69:16]
          "Default" [plain @70:5..70:12]: "false" [double-quoted @70:14..70:21]
        "pCreateAlias" [plain @72:3..72:15]: mapping [block]
          "Description" [plain @73:5..73:16]: literal [@73:18..75:1]
            "A unique alias to assign to the Microsoft Active Directory in AWS. AWS Directory Service uses the alias to construct the access URL for the directory, such as http://alias.awsapps.com. By default, AWS CloudFormation does not create an alias.\n"
          "Type" [plain @75:5..75:9]: "String" [plain @75:11..75:17]
          "AllowedValues" [plain @76:5..76:18]: sequence [block]
            - "true" [double-quoted @77:9..77:15]
            - "false" [double-quoted @78:9..78:16]
          "Default" [plain @79:5..79:12]: "false" [double-quoted @79:14..79:21]
        "pPrivateSubnet1" [plain @81:3..81:18]: mapping [block]
          "Description" [plain @82:5..82:16]: literal [@82:18..84:1]
            "A subnet within the selected VPC. Each subnet must be in different Availability Zones (AZs). AWS Directory Service creates a directory server and a DNS server in each subnet.\n"
          "Type" [plain @84:5..84:9]: "AWS::EC2::Subnet::Id" [plain @84:11..84:31]
        "pPrivateSubnet2" [plain @86:3..86:18]: mapping [block]
          "Description" [plain @87:5..87:16]: literal [@87:18..89:1]
            "A second subnet in same VPC that is in different AZ. Each subnet must be in different Availability Zones (AZs). AWS Directory Service creates a directory server and a DNS server in each subnet.\n"
          "Type" [plain @89:5..89:9]: "AWS::EC2::Subnet::Id" [plain @89:11..89:31]
        "pVPCID" [plain @91:3..91:9]: mapping [block]
          "Description" [plain @92:5..92:16]: literal [@92:18..94:1]
            "The VPC ID in which to create the Microsoft Active Directory server.\n"
          "Type" [plain @94:5..94:9]: "AWS::EC2::VPC::Id" [plain @94:11..94:28]
      "Conditions" [plain @96:1..96:11]: mapping [block]
        "cAlias" [plain @97:3..97:9]: sequence [block] !<!Equals>
          - "pCreateAlias" [plain @98:12..98:24] !<!Ref>
          - "true" [double-quoted @99:7..99:13]
      "Resources" [plain @101:1..101:10]: mapping [block]
        "rMSDirectory" [plain @102:3..102:15]: mapping [block]
          "Type" [plain @103:5..103:9]: "AWS::DirectoryService::MicrosoftAD" [plain @103:11..103:45]
          "Properties" [plain @104:5..104:15]: mapping [block]
            "CreateAlias" [plain @105:7..105:18]: "pCreateAlias" [plain @105:25..105:37] !<!Ref>
            "Edition" [plain @106:7..106:14]: "pEdition" [plain @106:21..106:29] !<!Ref>
            "EnableSso" [plain @107:7..107:16]: "pEnableSingleSignOn" [plain @107:23..107:42] !<!Ref>
            "Name" [plain @108:7..108:11]: "pDomainName" [plain @108:18..108:29] !<!Ref>
            "Password" [plain @109:7..109:15]: "{{resolve:secretsmanager:microsoft-ad-pw:SecretString:password}}" [single-quoted @109:17..109:83]
            "ShortName" [plain @110:7..110:16]: "pMicrosoftADShortName" [plain @110:23..110:44] !<!Ref>
            "VpcSettings" [plain @111:7..111:18]: mapping [block]
              "SubnetIds" [plain @112:9..112:18]: sequence [block]
                - "pPrivateSubnet1" [plain @113:18..113:33] !<!Ref>
                - "pPrivateSubnet2" [plain @114:18..114:33] !<!Ref>
              "VpcId" [plain @115:9..115:14]: "pVPCID" [plain @115:21..115:27] !<!Ref>
      "Outputs" [plain @117:1..117:8]: mapping [block]
        "DirectoryID" [plain @118:3..118:14]: mapping [block]
          "Description" [plain @119:5..119:16]: "ID of the MS Directory" [plain @119:18..119:40]
          "Value" [plain @120:5..120:10]: "rMSDirectory" [plain @120:17..120:29] !<!Ref>
        "PrimaryDNS" [plain @122:3..122:13]: mapping [block]
          "Description" [plain @123:5..123:16]: "DNS IPs of the MS Directory" [plain @123:18..123:45]
          "Value" [plain @124:5..124:10]: sequence [block] !<!Select>
            - "0" [plain @125:9..125:10]
            - "rMSDirectory.DnsIpAddresses" [plain @126:17..126:44] !<!GetAtt>
        "SecondaryDNS" [plain @128:3..128:15]: mapping [block]
          "Description" [plain @129:5..129:16]: "DNS IPs of the MSDirectory" [plain @129:18..129:44]
          "Value" [plain @130:5..130:10]: sequence [block] !<!Select>
            - "1" [plain @131:9..131:10]
            - "rMSDirectory.DnsIpAddresses" [plain @132:17..132:44] !<!GetAtt>
        "DirectoryAlias" [plain @134:3..134:17]: mapping [block]
          "Description" [plain @135:5..135:16]: "URL for the alias" [plain @135:18..135:35]
          "Value" [plain @136:5..136:10]: "rMSDirectory.Alias" [plain @136:20..136:38] !<!GetAtt>
          "Condition" [plain @137:5..137:14]: "cAlias" [plain @137:16..137:22]
    "#);
}
