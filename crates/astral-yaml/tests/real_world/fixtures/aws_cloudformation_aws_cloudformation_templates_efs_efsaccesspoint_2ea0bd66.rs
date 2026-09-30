#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-efs-efsaccesspoint-2ea0bd66"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Creates an EFS file system with three mount targets and one access point." [plain @3:14..3:87]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "EFSFileSystemName" [plain @6:3..6:20]: mapping [block]
          "Type" [plain @7:5..7:9]: "String" [plain @7:11..7:17]
          "Description" [plain @8:5..8:16]: "Name for the EFS file system" [plain @8:18..8:46]
          "Default" [plain @9:5..9:12]: "SampleEFSFilesystem" [plain @9:14..9:33]
        "AccessPointName" [plain @11:3..11:18]: mapping [block]
          "Type" [plain @12:5..12:9]: "String" [plain @12:11..12:17]
          "Description" [plain @13:5..13:16]: "Name for the EFS access point" [plain @13:18..13:47]
          "Default" [plain @14:5..14:12]: "SampleEFSAccessPoint" [plain @14:14..14:34]
        "Subnet1" [plain @16:3..16:10]: mapping [block]
          "Type" [plain @17:5..17:9]: "AWS::EC2::Subnet::Id" [plain @17:11..17:31]
          "Description" [plain @18:5..18:16]: "A subnet for the first EFS mount target" [plain @18:18..18:57]
        "Subnet2" [plain @20:3..20:10]: mapping [block]
          "Type" [plain @21:5..21:9]: "AWS::EC2::Subnet::Id" [plain @21:11..21:31]
          "Description" [plain @22:5..22:16]: "A subnet for the second EFS mount target" [plain @22:18..22:58]
        "Subnet3" [plain @24:3..24:10]: mapping [block]
          "Type" [plain @25:5..25:9]: "AWS::EC2::Subnet::Id" [plain @25:11..25:31]
          "Description" [plain @26:5..26:16]: "A subnet for the third EFS mount target" [plain @26:18..26:57]
        "SecurityGroup1" [plain @28:3..28:17]: mapping [block]
          "Type" [plain @29:5..29:9]: "AWS::EC2::SecurityGroup::Id" [plain @29:11..29:38]
          "Description" [plain @30:5..30:16]: "Security group for the first EFS mount target" [plain @30:18..30:63]
        "SecurityGroup2" [plain @32:3..32:17]: mapping [block]
          "Type" [plain @33:5..33:9]: "AWS::EC2::SecurityGroup::Id" [plain @33:11..33:38]
          "Description" [plain @34:5..34:16]: "Security group for the second EFS mount target" [plain @34:18..34:64]
        "SecurityGroup3" [plain @36:3..36:17]: mapping [block]
          "Type" [plain @37:5..37:9]: "AWS::EC2::SecurityGroup::Id" [plain @37:11..37:38]
          "Description" [plain @38:5..38:16]: "Security group for the third EFS mount target" [plain @38:18..38:63]
      "Resources" [plain @40:1..40:10]: mapping [block]
        "EFSFileSystem" [plain @41:3..41:16]: mapping [block]
          "Type" [plain @42:5..42:9]: "AWS::EFS::FileSystem" [plain @42:11..42:31]
          "Properties" [plain @43:5..43:15]: mapping [block]
            "Encrypted" [plain @44:7..44:16]: "true" [plain @44:18..44:22]
            "PerformanceMode" [plain @45:7..45:22]: "generalPurpose" [plain @45:24..45:38]
            "FileSystemTags" [plain @46:7..46:21]: sequence [block]
              - mapping [block]
                "Key" [plain @47:11..47:14]: "Name" [plain @47:16..47:20]
                "Value" [plain @48:11..48:16]: "EFSFileSystemName" [plain @48:23..48:40] !<!Ref>
        "EFSMountTarget1" [plain @50:3..50:18]: mapping [block]
          "Type" [plain @51:5..51:9]: "AWS::EFS::MountTarget" [plain @51:11..51:32]
          "Properties" [plain @52:5..52:15]: mapping [block]
            "FileSystemId" [plain @53:7..53:19]: "EFSFileSystem" [plain @53:26..53:39] !<!Ref>
            "SecurityGroups" [plain @54:7..54:21]: sequence [block]
              - "SecurityGroup1" [plain @55:16..55:30] !<!Ref>
            "SubnetId" [plain @56:7..56:15]: "Subnet1" [plain @56:22..56:29] !<!Ref>
        "EFSMountTarget2" [plain @58:3..58:18]: mapping [block]
          "Type" [plain @59:5..59:9]: "AWS::EFS::MountTarget" [plain @59:11..59:32]
          "Properties" [plain @60:5..60:15]: mapping [block]
            "FileSystemId" [plain @61:7..61:19]: "EFSFileSystem" [plain @61:26..61:39] !<!Ref>
            "SecurityGroups" [plain @62:7..62:21]: sequence [block]
              - "SecurityGroup2" [plain @63:16..63:30] !<!Ref>
            "SubnetId" [plain @64:7..64:15]: "Subnet2" [plain @64:22..64:29] !<!Ref>
        "EFSMountTarget3" [plain @66:3..66:18]: mapping [block]
          "Type" [plain @67:5..67:9]: "AWS::EFS::MountTarget" [plain @67:11..67:32]
          "Properties" [plain @68:5..68:15]: mapping [block]
            "FileSystemId" [plain @69:7..69:19]: "EFSFileSystem" [plain @69:26..69:39] !<!Ref>
            "SecurityGroups" [plain @70:7..70:21]: sequence [block]
              - "SecurityGroup3" [plain @71:16..71:30] !<!Ref>
            "SubnetId" [plain @72:7..72:15]: "Subnet3" [plain @72:22..72:29] !<!Ref>
        "EFSAccessPoint" [plain @74:3..74:17]: mapping [block]
          "Type" [plain @75:5..75:9]: "AWS::EFS::AccessPoint" [plain @75:11..75:32]
          "Properties" [plain @76:5..76:15]: mapping [block]
            "FileSystemId" [plain @77:7..77:19]: "EFSFileSystem" [plain @77:26..77:39] !<!Ref>
            "AccessPointTags" [plain @78:7..78:22]: sequence [block]
              - mapping [block]
                "Key" [plain @79:11..79:14]: "Name" [plain @79:16..79:20]
                "Value" [plain @80:11..80:16]: "AccessPointName" [plain @80:23..80:38] !<!Ref>
      "Outputs" [plain @82:1..82:8]: mapping [block]
        "FileSystemId" [plain @83:3..83:15]: mapping [block]
          "Description" [plain @84:5..84:16]: "ID of the created EFS File System" [plain @84:18..84:51]
          "Value" [plain @85:5..85:10]: "EFSFileSystem" [plain @85:17..85:30] !<!Ref>
    "#);
}
