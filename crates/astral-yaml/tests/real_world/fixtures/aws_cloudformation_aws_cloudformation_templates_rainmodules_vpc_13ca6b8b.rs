#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-rainmodules-vpc-13ca6b8b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "Resources" [plain @1:1..1:10]: mapping [block]
        "VPC" [plain @3:3..3:6]: mapping [block]
          "Type" [plain @4:5..4:9]: "AWS::EC2::VPC" [plain @4:11..4:24]
          "Properties" [plain @5:5..5:15]: mapping [block]
            "CidrBlock" [plain @6:7..6:16]: "10.0.0.0/16" [plain @6:18..6:29]
            "EnableDnsHostnames" [plain @7:7..7:25]: "true" [plain @7:27..7:31]
            "EnableDnsSupport" [plain @8:7..8:23]: "true" [plain @8:25..8:29]
            "InstanceTenancy" [plain @9:7..9:22]: "default" [plain @9:24..9:31]
        "PublicSubnet1" [plain @11:3..11:16]: mapping [block]
          "Type" [plain @12:5..12:9]: "AWS::EC2::Subnet" [plain @12:11..12:27]
          "Metadata" [plain @13:5..13:13]: mapping [block]
            "guard" [plain @14:7..14:12]: mapping [block]
              "SuppressedRules" [plain @15:9..15:24]: sequence [block]
                - "SUBNET_AUTO_ASSIGN_PUBLIC_IP_DISABLED" [plain @16:13..16:50]
          "Properties" [plain @17:5..17:15]: mapping [block]
            "AvailabilityZone" [plain @18:7..18:23]: sequence [flow] !<!Select>
              - "0" [plain @18:34..18:35]
              - mapping [flow]
                "Fn::GetAZs" [plain @18:37..18:47]: "AWS::Region" [double-quoted @18:54..18:67] !<!Ref>
            "CidrBlock" [plain @19:7..19:16]: "10.0.0.0/18" [plain @19:18..19:29]
            "MapPublicIpOnLaunch" [plain @20:7..20:26]: "true" [plain @20:28..20:32]
            "VpcId" [plain @21:7..21:12]: "VPC" [plain @21:19..21:22] !<!Ref>
        "PublicSubnet1RouteTable" [plain @23:3..23:26]: mapping [block]
          "Type" [plain @24:5..24:9]: "AWS::EC2::RouteTable" [plain @24:11..24:31]
          "Properties" [plain @25:5..25:15]: mapping [block]
            "VpcId" [plain @26:7..26:12]: "VPC" [plain @26:19..26:22] !<!Ref>
        "PublicSubnet1RouteTableAssociation" [plain @28:3..28:37]: mapping [block]
          "Type" [plain @29:5..29:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @29:11..29:48]
          "Properties" [plain @30:5..30:15]: mapping [block]
            "RouteTableId" [plain @31:7..31:19]: "PublicSubnet1RouteTable" [plain @31:26..31:49] !<!Ref>
            "SubnetId" [plain @32:7..32:15]: "PublicSubnet1" [plain @32:22..32:35] !<!Ref>
        "PublicSubnet1DefaultRoute" [plain @34:3..34:28]: mapping [block]
          "Type" [plain @35:5..35:9]: "AWS::EC2::Route" [plain @35:11..35:26]
          "Metadata" [plain @36:5..36:13]: mapping [block]
            "guard" [plain @37:7..37:12]: mapping [block]
              "SuppressedRules" [plain @38:9..38:24]: sequence [block]
                - "NO_UNRESTRICTED_ROUTE_TO_IGW" [plain @39:13..39:41]
          "Properties" [plain @40:5..40:15]: mapping [block]
            "DestinationCidrBlock" [plain @41:7..41:27]: "0.0.0.0/0" [plain @41:29..41:38]
            "GatewayId" [plain @42:7..42:16]: "InternetGateway" [plain @42:23..42:38] !<!Ref>
            "RouteTableId" [plain @43:7..43:19]: "PublicSubnet1RouteTable" [plain @43:26..43:49] !<!Ref>
          "DependsOn" [plain @44:5..44:14]: "VPCGW" [plain @44:16..44:21]
        "PublicSubnet1EIP" [plain @46:3..46:19]: mapping [block]
          "Type" [plain @47:5..47:9]: "AWS::EC2::EIP" [plain @47:11..47:24]
          "Properties" [plain @48:5..48:15]: mapping [block]
            "Domain" [plain @49:7..49:13]: "vpc" [plain @49:15..49:18]
        "PublicSubnet1NATGateway" [plain @51:3..51:26]: mapping [block]
          "Type" [plain @52:5..52:9]: "AWS::EC2::NatGateway" [plain @52:11..52:31]
          "Properties" [plain @53:5..53:15]: mapping [block]
            "AllocationId" [plain @54:7..54:19]: "PublicSubnet1EIP.AllocationId" [plain @54:29..54:58] !<!GetAtt>
            "SubnetId" [plain @55:7..55:15]: "PublicSubnet1" [plain @55:22..55:35] !<!Ref>
          "DependsOn" [plain @56:5..56:14]: sequence [block]
            - "PublicSubnet1DefaultRoute" [plain @57:9..57:34]
            - "PublicSubnet1RouteTableAssociation" [plain @58:9..58:43]
        "PublicSubnet2" [plain @60:3..60:16]: mapping [block]
          "Type" [plain @61:5..61:9]: "AWS::EC2::Subnet" [plain @61:11..61:27]
          "Metadata" [plain @62:5..62:13]: mapping [block]
            "guard" [plain @63:7..63:12]: mapping [block]
              "SuppressedRules" [plain @64:9..64:24]: sequence [block]
                - "SUBNET_AUTO_ASSIGN_PUBLIC_IP_DISABLED" [plain @65:13..65:50]
          "Properties" [plain @66:5..66:15]: mapping [block]
            "AvailabilityZone" [plain @67:7..67:23]: sequence [flow] !<!Select>
              - "1" [plain @67:34..67:35]
              - mapping [flow]
                "Fn::GetAZs" [plain @67:37..67:47]: "AWS::Region" [double-quoted @67:54..67:67] !<!Ref>
            "CidrBlock" [plain @68:7..68:16]: "10.0.64.0/18" [plain @68:18..68:30]
            "MapPublicIpOnLaunch" [plain @69:7..69:26]: "true" [plain @69:28..69:32]
            "VpcId" [plain @70:7..70:12]: "VPC" [plain @70:19..70:22] !<!Ref>
        "PublicSubnet2RouteTable" [plain @72:3..72:26]: mapping [block]
          "Type" [plain @73:5..73:9]: "AWS::EC2::RouteTable" [plain @73:11..73:31]
          "Properties" [plain @74:5..74:15]: mapping [block]
            "VpcId" [plain @75:7..75:12]: "VPC" [plain @75:19..75:22] !<!Ref>
        "PublicSubnet2RouteTableAssociation" [plain @77:3..77:37]: mapping [block]
          "Type" [plain @78:5..78:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @78:11..78:48]
          "Properties" [plain @79:5..79:15]: mapping [block]
            "RouteTableId" [plain @80:7..80:19]: "PublicSubnet2RouteTable" [plain @80:26..80:49] !<!Ref>
            "SubnetId" [plain @81:7..81:15]: "PublicSubnet2" [plain @81:22..81:35] !<!Ref>
        "PublicSubnet2DefaultRoute" [plain @83:3..83:28]: mapping [block]
          "Type" [plain @84:5..84:9]: "AWS::EC2::Route" [plain @84:11..84:26]
          "Metadata" [plain @85:5..85:13]: mapping [block]
            "guard" [plain @86:7..86:12]: mapping [block]
              "SuppressedRules" [plain @87:9..87:24]: sequence [block]
                - "NO_UNRESTRICTED_ROUTE_TO_IGW" [plain @88:13..88:41]
          "Properties" [plain @89:5..89:15]: mapping [block]
            "DestinationCidrBlock" [plain @90:7..90:27]: "0.0.0.0/0" [plain @90:29..90:38]
            "GatewayId" [plain @91:7..91:16]: "InternetGateway" [plain @91:23..91:38] !<!Ref>
            "RouteTableId" [plain @92:7..92:19]: "PublicSubnet2RouteTable" [plain @92:26..92:49] !<!Ref>
          "DependsOn" [plain @93:5..93:14]: "VPCGW" [plain @93:16..93:21]
        "PublicSubnet2EIP" [plain @95:3..95:19]: mapping [block]
          "Type" [plain @96:5..96:9]: "AWS::EC2::EIP" [plain @96:11..96:24]
          "Properties" [plain @97:5..97:15]: mapping [block]
            "Domain" [plain @98:7..98:13]: "vpc" [plain @98:15..98:18]
        "PublicSubnet2NATGateway" [plain @100:3..100:26]: mapping [block]
          "Type" [plain @101:5..101:9]: "AWS::EC2::NatGateway" [plain @101:11..101:31]
          "Properties" [plain @102:5..102:15]: mapping [block]
            "AllocationId" [plain @103:7..103:19]: "PublicSubnet2EIP.AllocationId" [plain @103:29..103:58] !<!GetAtt>
            "SubnetId" [plain @104:7..104:15]: "PublicSubnet2" [plain @104:22..104:35] !<!Ref>
          "DependsOn" [plain @105:5..105:14]: sequence [block]
            - "PublicSubnet2DefaultRoute" [plain @106:9..106:34]
            - "PublicSubnet2RouteTableAssociation" [plain @107:9..107:43]
        "PrivateSubnet1Subnet" [plain @109:3..109:23]: mapping [block]
          "Type" [plain @110:5..110:9]: "AWS::EC2::Subnet" [plain @110:11..110:27]
          "Properties" [plain @111:5..111:15]: mapping [block]
            "AvailabilityZone" [plain @112:7..112:23]: sequence [flow] !<!Select>
              - "0" [plain @112:34..112:35]
              - mapping [flow]
                "Fn::GetAZs" [plain @112:37..112:47]: "AWS::Region" [double-quoted @112:54..112:67] !<!Ref>
            "CidrBlock" [plain @113:7..113:16]: "10.0.128.0/18" [plain @113:18..113:31]
            "MapPublicIpOnLaunch" [plain @114:7..114:26]: "false" [plain @114:28..114:33]
            "VpcId" [plain @115:7..115:12]: "VPC" [plain @115:19..115:22] !<!Ref>
        "PrivateSubnet1RouteTable" [plain @117:3..117:27]: mapping [block]
          "Type" [plain @118:5..118:9]: "AWS::EC2::RouteTable" [plain @118:11..118:31]
          "Properties" [plain @119:5..119:15]: mapping [block]
            "VpcId" [plain @120:7..120:12]: "VPC" [plain @120:19..120:22] !<!Ref>
        "PrivateSubnet1RouteTableAssociation" [plain @122:3..122:38]: mapping [block]
          "Type" [plain @123:5..123:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @123:11..123:48]
          "Properties" [plain @124:5..124:15]: mapping [block]
            "RouteTableId" [plain @125:7..125:19]: "PrivateSubnet1RouteTable" [plain @125:26..125:50] !<!Ref>
            "SubnetId" [plain @126:7..126:15]: "PrivateSubnet1Subnet" [plain @126:22..126:42] !<!Ref>
        "PrivateSubnet1DefaultRoute" [plain @128:3..128:29]: mapping [block]
          "Type" [plain @129:5..129:9]: "AWS::EC2::Route" [plain @129:11..129:26]
          "Properties" [plain @130:5..130:15]: mapping [block]
            "DestinationCidrBlock" [plain @131:7..131:27]: "0.0.0.0/0" [plain @131:29..131:38]
            "NatGatewayId" [plain @132:7..132:19]: "PublicSubnet1NATGateway" [plain @132:26..132:49] !<!Ref>
            "RouteTableId" [plain @133:7..133:19]: "PrivateSubnet1RouteTable" [plain @133:26..133:50] !<!Ref>
        "PrivateSubnet2Subnet" [plain @135:3..135:23]: mapping [block]
          "Type" [plain @136:5..136:9]: "AWS::EC2::Subnet" [plain @136:11..136:27]
          "Properties" [plain @137:5..137:15]: mapping [block]
            "AvailabilityZone" [plain @138:7..138:23]: sequence [flow] !<!Select>
              - "1" [plain @138:34..138:35]
              - mapping [flow]
                "Fn::GetAZs" [plain @138:37..138:47]: "AWS::Region" [double-quoted @138:54..138:67] !<!Ref>
            "CidrBlock" [plain @139:7..139:16]: "10.0.192.0/18" [plain @139:18..139:31]
            "MapPublicIpOnLaunch" [plain @140:7..140:26]: "false" [plain @140:28..140:33]
            "VpcId" [plain @141:7..141:12]: "VPC" [plain @141:19..141:22] !<!Ref>
        "PrivateSubnet2RouteTable" [plain @143:3..143:27]: mapping [block]
          "Type" [plain @144:5..144:9]: "AWS::EC2::RouteTable" [plain @144:11..144:31]
          "Properties" [plain @145:5..145:15]: mapping [block]
            "VpcId" [plain @146:7..146:12]: "VPC" [plain @146:19..146:22] !<!Ref>
        "PrivateSubnet2RouteTableAssociation" [plain @148:3..148:38]: mapping [block]
          "Type" [plain @149:5..149:9]: "AWS::EC2::SubnetRouteTableAssociation" [plain @149:11..149:48]
          "Properties" [plain @150:5..150:15]: mapping [block]
            "RouteTableId" [plain @151:7..151:19]: "PrivateSubnet2RouteTable" [plain @151:26..151:50] !<!Ref>
            "SubnetId" [plain @152:7..152:15]: "PrivateSubnet2Subnet" [plain @152:22..152:42] !<!Ref>
        "PrivateSubnet2DefaultRoute" [plain @154:3..154:29]: mapping [block]
          "Type" [plain @155:5..155:9]: "AWS::EC2::Route" [plain @155:11..155:26]
          "Properties" [plain @156:5..156:15]: mapping [block]
            "DestinationCidrBlock" [plain @157:7..157:27]: "0.0.0.0/0" [plain @157:29..157:38]
            "NatGatewayId" [plain @158:7..158:19]: "PublicSubnet2NATGateway" [plain @158:26..158:49] !<!Ref>
            "RouteTableId" [plain @159:7..159:19]: "PrivateSubnet2RouteTable" [plain @159:26..159:50] !<!Ref>
        "InternetGateway" [plain @161:3..161:18]: mapping [block]
          "Type" [plain @162:5..162:9]: "AWS::EC2::InternetGateway" [plain @162:11..162:36]
        "VPCGW" [plain @164:3..164:8]: mapping [block]
          "Type" [plain @165:5..165:9]: "AWS::EC2::VPCGatewayAttachment" [plain @165:11..165:41]
          "Properties" [plain @166:5..166:15]: mapping [block]
            "InternetGatewayId" [plain @167:7..167:24]: "InternetGateway" [plain @167:31..167:46] !<!Ref>
            "VpcId" [plain @168:7..168:12]: "VPC" [plain @168:19..168:22] !<!Ref>
    "#);
}
