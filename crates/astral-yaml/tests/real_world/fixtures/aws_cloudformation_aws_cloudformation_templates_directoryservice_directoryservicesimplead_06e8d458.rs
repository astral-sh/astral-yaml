#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-directoryservice-directoryservicesimplead-06e8d458"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Parameters" [plain @3:1..3:11]: mapping [block]
        "DomainName" [plain @4:3..4:13]: mapping [block]
          "Description" [plain @5:5..5:16]: "FQDN of the domain for this directory" [plain @5:18..5:55]
          "Type" [plain @6:5..6:9]: "String" [plain @6:11..6:17]
          "Default" [plain @7:5..7:12]: "corp.example.com" [plain @7:14..7:30]
        "SimpleADShortName" [plain @9:3..9:20]: mapping [block]
          "Description" [plain @10:5..10:16]: "Netbios name of the domain for this directory" [plain @10:18..10:63]
          "Type" [plain @11:5..11:9]: "String" [plain @11:11..11:17]
          "Default" [plain @12:5..12:12]: "corp" [plain @12:14..12:18]
        "CreateAlias" [plain @14:3..14:14]: mapping [block]
          "Description" [plain @15:5..15:16]: "Only required for applications which need a URL to connect to the directory" [plain @15:18..15:93]
          "Type" [plain @16:5..16:9]: "String" [plain @16:11..16:17]
          "AllowedValues" [plain @17:5..17:18]: sequence [block]
            - "true" [double-quoted @18:9..18:15]
            - "false" [double-quoted @19:9..19:16]
          "Default" [plain @20:5..20:12]: "false" [double-quoted @20:14..20:21]
        "PrivateSubnet1" [plain @22:3..22:17]: mapping [block]
          "Description" [plain @23:5..23:16]: "Subnet to be used for the Directoty" [plain @23:18..23:53]
          "Type" [plain @24:5..24:9]: "List<AWS::EC2::Subnet::Id>" [plain @24:11..24:37]
        "PrivateSubnet2" [plain @26:3..26:17]: mapping [block]
          "Description" [plain @27:5..27:16]: "Subnet to be used for the Directoty" [plain @27:18..27:53]
          "Type" [plain @28:5..28:9]: "List<AWS::EC2::Subnet::Id>" [plain @28:11..28:37]
        "VPCID" [plain @30:3..30:8]: mapping [block]
          "Description" [plain @31:5..31:16]: "The VPC the directory will be created in" [plain @31:18..31:58]
          "Type" [plain @32:5..32:9]: "List<AWS::EC2::VPC::Id>" [plain @32:11..32:34]
        "Size" [plain @34:3..34:7]: mapping [block]
          "Description" [plain @35:5..35:16]: "Size of the Simple AD" [plain @35:18..35:39]
          "Type" [plain @36:5..36:9]: "String" [plain @36:11..36:17]
          "AllowedValues" [plain @37:5..37:18]: sequence [block]
            - "Small" [plain @38:9..38:14]
            - "Large" [plain @39:9..39:14]
          "Default" [plain @40:5..40:12]: "Small" [plain @40:14..40:19]
      "Conditions" [plain @42:1..42:11]: mapping [block]
        "Alias" [plain @43:3..43:8]: sequence [block] !<!Equals>
          - "CreateAlias" [plain @44:12..44:23] !<!Ref>
          - "true" [double-quoted @45:7..45:13]
      "Resources" [plain @47:1..47:10]: mapping [block]
        "SimpleAD" [plain @48:3..48:11]: mapping [block]
          "Type" [plain @49:5..49:9]: "AWS::DirectoryService::SimpleAD" [plain @49:11..49:42]
          "Properties" [plain @50:5..50:15]: mapping [block]
            "CreateAlias" [plain @51:7..51:18]: "false" [plain @51:20..51:25]
            "EnableSso" [plain @52:7..52:16]: "false" [plain @52:18..52:23]
            "Name" [plain @53:7..53:11]: "DomainName" [plain @53:18..53:28] !<!Ref>
            "Password" [plain @54:7..54:15]: "{{resolve:secretsmanager:simple-ad-pw:SecretString:pasword}}" [single-quoted @54:17..54:79]
            "ShortName" [plain @55:7..55:16]: "SimpleADShortName" [plain @55:23..55:40] !<!Ref>
            "Size" [plain @56:7..56:11]: "Size" [plain @56:18..56:22] !<!Ref>
            "VpcSettings" [plain @57:7..57:18]: mapping [block]
              "SubnetIds" [plain @58:9..58:18]: sequence [block]
                - sequence [block] !<!Select>
                  - "0" [double-quoted @60:15..60:18]
                  - "PrivateSubnet1" [plain @61:20..61:34] !<!Ref>
                - sequence [block] !<!Select>
                  - "0" [double-quoted @63:15..63:18]
                  - "PrivateSubnet2" [plain @64:20..64:34] !<!Ref>
              "VpcId" [plain @65:9..65:14]: sequence [block] !<!Select>
                - "0" [double-quoted @66:13..66:16]
                - "VPCID" [plain @67:18..67:23] !<!Ref>
      "Outputs" [plain @69:1..69:8]: mapping [block]
        "DirectoryID" [plain @70:3..70:14]: mapping [block]
          "Description" [plain @71:5..71:16]: "ID of the SimpleAD" [plain @71:18..71:36]
          "Value" [plain @72:5..72:10]: "SimpleAD" [plain @72:17..72:25] !<!Ref>
        "PrimaryDNS" [plain @74:3..74:13]: mapping [block]
          "Description" [plain @75:5..75:16]: "DNS IPs of the SimpleAD" [plain @75:18..75:41]
          "Value" [plain @76:5..76:10]: sequence [block] !<!Select>
            - "0" [plain @77:9..77:10]
            - "SimpleAD.DnsIpAddresses" [plain @78:17..78:40] !<!GetAtt>
        "SecondaryDNS" [plain @80:3..80:15]: mapping [block]
          "Description" [plain @81:5..81:16]: "DNS IPs of the SimpleAD" [plain @81:18..81:41]
          "Value" [plain @82:5..82:10]: sequence [block] !<!Select>
            - "1" [plain @83:9..83:10]
            - "SimpleAD.DnsIpAddresses" [plain @84:17..84:40] !<!GetAtt>
        "DirectoryAlias" [plain @86:3..86:17]: mapping [block]
          "Description" [plain @87:5..87:16]: "URL for the alias" [plain @87:18..87:35]
          "Value" [plain @88:5..88:10]: "SimpleAD.Alias" [plain @88:20..88:34] !<!GetAtt>
          "Condition" [plain @89:5..89:14]: "Alias" [plain @89:16..89:21]
    "#);
}
