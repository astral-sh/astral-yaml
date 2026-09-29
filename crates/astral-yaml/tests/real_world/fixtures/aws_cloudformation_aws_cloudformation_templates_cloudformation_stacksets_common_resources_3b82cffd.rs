#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("aws-cloudformation-aws-cloudformation-templates-cloudformation-stacksets-common-resources-3b82cffd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "Description" [plain @1:1..1:12]: literal [@1:14..9:1]
        "This template has resources that will be installed into all managed accounts\n"
        "in the OU. For the purposes of the sample it's not important what exactly we\n"
        "are creating here. To demonstrate the consolidated logging, errors can be\n"
        "introduced into this template, such as choosing a bucket name that already\n"
        "exists. This template uses a Rain module, which can be packaged with `rain\n"
        "pkg -x common-resources.yaml`.\n"
      "Parameters" [plain @9:1..9:11]: mapping [block]
        "AppName" [plain @10:3..10:10]: mapping [block]
          "Type" [plain @11:5..11:9]: "String" [plain @11:11..11:17]
          "Description" [plain @12:5..12:16]: "This name will be used as part of resource names" [plain @12:18..12:66]
          "Default" [plain @13:5..13:12]: "stacksets-sample" [plain @13:14..13:30]
      "Resources" [plain @15:1..15:10]: mapping [block]
        "Storage" [plain @16:3..16:10]: mapping [block]
          "Type" [plain @17:5..17:9]: "../../RainModules/bucket.yml" [plain @17:25..17:53] !<!Rain::Module>
          "Properties" [plain @18:5..18:15]: mapping [block]
            "AppName" [plain @19:7..19:14]: "AppName" [plain @19:21..19:28] !<!Ref>
        "TestQ" [plain @21:3..21:8]: mapping [block]
          "Type" [plain @22:5..22:9]: "AWS::SQS::Queue" [plain @22:11..22:26]
          "Properties" [plain @23:5..23:15]: mapping [block]
            "QueueName" [plain @24:7..24:16]: "test-events17" [plain @24:18..24:31]
    "#);
}
