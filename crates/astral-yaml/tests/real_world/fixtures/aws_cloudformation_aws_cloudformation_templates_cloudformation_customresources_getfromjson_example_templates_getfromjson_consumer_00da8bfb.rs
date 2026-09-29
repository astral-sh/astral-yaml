#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-cloudformation-customresources-getfromjson-example-templates-getfromjson-consumer-00da8bfb"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "AWSTemplateFormatVersion" [plain @1:1..1:25]: "2010-09-09" [double-quoted @1:27..1:39]
      "Description" [plain @3:1..3:12]: "This AWS CloudFormation template describes a sample CloudFormation custom resource consumer for the GetFromJson Lambda-backed custom resource provider." [plain @3:14..3:165]
      "Parameters" [plain @5:1..5:11]: mapping [block]
        "GetFromListJsonData" [plain @6:3..6:22]: mapping [block]
          "Description" [plain @7:5..7:16]: "Example JSON data representing a list of values." [plain @7:18..7:66]
          "Type" [plain @8:5..8:9]: "String" [plain @8:11..8:17]
          "Default" [plain @9:5..9:12]: "[\"test0\", \"test1\", \"test2\"]" [single-quoted @9:14..9:43]
        "GetFromListJsonDataQuery" [plain @11:3..11:27]: mapping [block]
          "Description" [plain @12:5..12:16]: "Example query for JSON data representing a list of values." [plain @12:18..12:76]
          "Type" [plain @13:5..13:9]: "String" [plain @13:11..13:17]
          "Default" [plain @14:5..14:12]: "[2]" [single-quoted @14:14..14:19]
        "GetFromMapJsonData" [plain @16:3..16:21]: mapping [block]
          "Description" [plain @17:5..17:16]: "Example JSON data representing a map data structure." [plain @17:18..17:70]
          "Type" [plain @18:5..18:9]: "String" [plain @18:11..18:17]
          "Default" [plain @19:5..19:12]: "{\"test\": {\"test1\": [\"x\", \"y\"]}}" [single-quoted @19:14..19:47]
        "GetFromMapJsonDataQuery" [plain @21:3..21:26]: mapping [block]
          "Description" [plain @22:5..22:16]: "Example query for JSON data representing a map data structure." [plain @22:18..22:80]
          "Type" [plain @23:5..23:9]: "String" [plain @23:11..23:17]
          "Default" [plain @24:5..24:12]: "[\"test\"][\"test1\"][1]" [single-quoted @24:14..24:36]
      "Resources" [plain @26:1..26:10]: mapping [block]
        "GetFromJsonCustomResourceSampleGetFromList" [plain @27:3..27:45]: mapping [block]
          "Type" [plain @28:5..28:9]: "Custom::GetFromJson" [plain @28:11..28:30]
          "Properties" [plain @29:5..29:15]: mapping [block]
            "ServiceTimeout" [plain @30:7..30:21]: "1" [plain @30:23..30:24]
            "ServiceToken" [plain @31:7..31:19]: "Custom-GetFromJson" [plain @31:34..31:52] !<!ImportValue>
            "json_data" [plain @32:7..32:16]: "GetFromListJsonData" [plain @32:23..32:42] !<!Ref>
            "search" [plain @33:7..33:13]: "GetFromListJsonDataQuery" [plain @33:20..33:44] !<!Ref>
        "GetFromJsonCustomResourceSampleGetFromMap" [plain @35:3..35:44]: mapping [block]
          "Type" [plain @36:5..36:9]: "Custom::GetFromJson" [plain @36:11..36:30]
          "Properties" [plain @37:5..37:15]: mapping [block]
            "ServiceTimeout" [plain @38:7..38:21]: "1" [plain @38:23..38:24]
            "ServiceToken" [plain @39:7..39:19]: "Custom-GetFromJson" [plain @39:34..39:52] !<!ImportValue>
            "json_data" [plain @40:7..40:16]: "GetFromMapJsonData" [plain @40:23..40:41] !<!Ref>
            "search" [plain @41:7..41:13]: "GetFromMapJsonDataQuery" [plain @41:20..41:43] !<!Ref>
      "Outputs" [plain @43:1..43:8]: mapping [block]
        "GetFromJsonCustomResourceSampleGetFromListValue" [plain @44:3..44:50]: mapping [block]
          "Value" [plain @45:5..45:10]: "GetFromJsonCustomResourceSampleGetFromList.Data" [plain @45:20..45:67] !<!GetAtt>
        "GetFromJsonCustomResourceSampleGetFromMapValue" [plain @47:3..47:49]: mapping [block]
          "Value" [plain @48:5..48:10]: "GetFromJsonCustomResourceSampleGetFromMap.Data" [plain @48:20..48:66] !<!GetAtt>
    "#);
}
