#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-elasticloadbalancing-networkloadbalancerwitheips-20db62e8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "Template creates a Network Load Balancer in 2 AZs with EIPs listening on TCP port 80. There are no registered targets these would either be EC2 instance IDs added to the targets property of the target group  or defined under the autoscaling group resources  " [single-quoted @3:14..3:274]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "VPC" [plain @6:3..6:6]: mapping [block]
          "Type" [plain @7:5..7:9]: "List<AWS::EC2::VPC::Id>" [plain @7:11..7:34]
        "Subnet1" [plain @9:3..9:10]: mapping [block]
          "Description" [plain @10:5..10:16]: "ID of the Subnet the instance should be launched in, this will link the instance to the same VPC." [plain @10:18..10:115]
          "Type" [plain @11:5..11:9]: "List<AWS::EC2::Subnet::Id>" [plain @11:11..11:37]
        "Subnet2" [plain @13:3..13:10]: mapping [block]
          "Description" [plain @14:5..14:16]: "ID of the Subnet the instance should be launched in, this will link the instance to the same VPC." [plain @14:18..14:115]
          "Type" [plain @15:5..15:9]: "List<AWS::EC2::Subnet::Id>" [plain @15:11..15:37]
        "ELBType" [plain @17:3..17:10]: mapping [block]
          "Type" [plain @18:5..18:9]: "String" [plain @18:11..18:17]
          "Default" [plain @19:5..19:12]: "network" [plain @19:14..19:21]
        "ELBIpAddressType" [plain @21:3..21:19]: mapping [block]
          "Type" [plain @22:5..22:9]: "String" [plain @22:11..22:17]
          "AllowedValues" [plain @23:5..23:18]: sequence [block]
            - "ipv4" [plain @24:9..24:13]
            - "dualstack" [plain @25:9..25:18]
          "Default" [plain @26:5..26:12]: "ipv4" [plain @26:14..26:18]
      "Resources" [plain @28:1..28:10]: mapping [block]
        "EIP1" [plain @29:3..29:7]: mapping [block]
          "Type" [plain @30:5..30:9]: "AWS::EC2::EIP" [plain @30:11..30:24]
          "Properties" [plain @31:5..31:15]: mapping [block]
            "Domain" [plain @32:7..32:13]: "vpc" [plain @32:15..32:18]
        "EIP2" [plain @34:3..34:7]: mapping [block]
          "Type" [plain @35:5..35:9]: "AWS::EC2::EIP" [plain @35:11..35:24]
          "Properties" [plain @36:5..36:15]: mapping [block]
            "Domain" [plain @37:7..37:13]: "vpc" [plain @37:15..37:18]
        "loadBalancer" [plain @39:3..39:15]: mapping [block]
          "Type" [plain @40:5..40:9]: "AWS::ElasticLoadBalancingV2::LoadBalancer" [plain @40:11..40:52]
          "DependsOn" [plain @41:5..41:14]: sequence [block]
            - "EIP2" [plain @42:9..42:13]
            - "EIP1" [plain @43:9..43:13]
          "Properties" [plain @44:5..44:15]: mapping [block]
            "SubnetMappings" [plain @45:7..45:21]: sequence [block]
              - mapping [block]
                "AllocationId" [plain @46:11..46:23]: "EIP1.AllocationId" [plain @46:33..46:50] !<!GetAtt>
                "SubnetId" [plain @47:11..47:19]: sequence [block] !<!Select>
                  - "0" [double-quoted @48:15..48:18]
                  - "Subnet1" [plain @49:20..49:27] !<!Ref>
              - mapping [block]
                "AllocationId" [plain @50:11..50:23]: "EIP2.AllocationId" [plain @50:33..50:50] !<!GetAtt>
                "SubnetId" [plain @51:11..51:19]: sequence [block] !<!Select>
                  - "0" [double-quoted @52:15..52:18]
                  - "Subnet2" [plain @53:20..53:27] !<!Ref>
            "Type" [plain @54:7..54:11]: "ELBType" [plain @54:18..54:25] !<!Ref>
            "IpAddressType" [plain @55:7..55:20]: "ELBIpAddressType" [plain @55:27..55:43] !<!Ref>
        "FirstEIP" [plain @57:3..57:11]: mapping [block]
          "Type" [plain @58:5..58:9]: "AWS::EC2::EIP" [plain @58:11..58:24]
          "Properties" [plain @59:5..59:15]: mapping [block]
            "Domain" [plain @60:7..60:13]: "vpc" [plain @60:15..60:18]
        "SecondEIP" [plain @62:3..62:12]: mapping [block]
          "Type" [plain @63:5..63:9]: "AWS::EC2::EIP" [plain @63:11..63:24]
          "Properties" [plain @64:5..64:15]: mapping [block]
            "Domain" [plain @65:7..65:13]: "vpc" [plain @65:15..65:18]
        "TargetGroup" [plain @67:3..67:14]: mapping [block]
          "Type" [plain @68:5..68:9]: "AWS::ElasticLoadBalancingV2::TargetGroup" [plain @68:11..68:51]
          "Properties" [plain @69:5..69:15]: mapping [block]
            "Name" [plain @70:7..70:11]: "MyTargets" [plain @70:13..70:22]
            "Port" [plain @71:7..71:11]: "10" [plain @71:13..71:15]
            "Protocol" [plain @72:7..72:15]: "TCP" [plain @72:17..72:20]
            "TargetGroupAttributes" [plain @73:7..73:28]: sequence [block]
              - mapping [block]
                "Key" [plain @74:11..74:14]: "deregistration_delay.timeout_seconds" [plain @74:16..74:52]
                "Value" [plain @75:11..75:16]: "20" [double-quoted @75:18..75:22]
            "VpcId" [plain @76:7..76:12]: sequence [block] !<!Select>
              - "0" [double-quoted @77:11..77:14]
              - "VPC" [plain @78:16..78:19] !<!Ref>
        "Listener" [plain @80:3..80:11]: mapping [block]
          "Type" [plain @81:5..81:9]: "AWS::ElasticLoadBalancingV2::Listener" [plain @81:11..81:48]
          "Properties" [plain @82:5..82:15]: mapping [block]
            "DefaultActions" [plain @83:7..83:21]: sequence [block]
              - mapping [block]
                "Type" [plain @84:11..84:15]: "forward" [plain @84:17..84:24]
                "TargetGroupArn" [plain @85:11..85:25]: "TargetGroup" [plain @85:32..85:43] !<!Ref>
            "LoadBalancerArn" [plain @86:7..86:22]: "loadBalancer" [plain @86:29..86:41] !<!Ref>
            "Port" [plain @87:7..87:11]: "80" [double-quoted @87:13..87:17]
            "Protocol" [plain @88:7..88:15]: "TCP" [plain @88:17..88:20]
    "#);
}
