#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("matrix-receipt"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "$schema" [plain @1:1..1:8]: "https://json-schema.org/draft/2020-12/schema" [plain @1:10..1:54]
      "type" [plain @3:1..3:5]: "object" [plain @3:7..3:13]
      "title" [plain @4:1..4:6]: "Receipt Event" [plain @4:8..4:21]
      "description" [plain @5:1..5:12]: "Informs the client of new receipts." [plain @5:14..5:49]
      "x-changedInMatrixVersion" [plain @6:1..6:25]: mapping [block]
        "1.4" [double-quoted @7:5..7:10]: literal [@7:12..9:1]
          "Added `m.read.private` receipts to the event's `content`.\n"
      "allOf" [plain @9:1..9:6]: sequence [block]
        - mapping [block]
          "$ref" [plain @10:7..10:11]: "core-event-schema/event.yaml" [double-quoted @10:13..10:43]
      "properties" [plain @11:1..11:11]: mapping [block]
        "content" [plain @12:5..12:12]: mapping [block]
          "type" [plain @13:9..13:13]: "object" [plain @13:15..13:21]
          "description" [plain @14:9..14:20]: literal [@14:22..18:1]
            "The mapping of event ID to a collection of receipts for this\n"
            "event ID. The event ID is the ID of the event being acknowledged\n"
            "and *not* an ID for the receipt itself."
          "patternProperties" [plain @18:9..18:26]: mapping [block]
            "^\\$" [double-quoted @19:13..19:19]: mapping [block]
              "type" [plain @20:17..20:21]: "object" [plain @20:23..20:29]
              "x-pattern-format" [plain @21:17..21:33]: "mx-event-id" [double-quoted @21:35..21:48]
              "title" [plain @22:17..22:22]: "Event Receipts" [plain @22:24..22:38]
              "description" [plain @23:17..23:28]: "The collection of receipts for this event ID." [literal @23:30..25:1]
              "properties" [plain @25:17..25:27]: mapping [block]
                "m.read" [double-quoted @26:21..26:29]: mapping [block]
                  "type" [plain @27:25..27:29]: "object" [plain @27:31..27:37]
                  "description" [plain @28:25..28:36]: literal [@28:38..32:1]
                    "A collection of users who have sent `m.read` receipts for\n"
                    "this event. The string key is the user ID the receipt\n"
                    "belongs to."
                  "patternProperties" [plain @32:25..32:42]: mapping [block]
                    "^@" [double-quoted @33:29..33:33]: mapping [block] &receiptUserMap
                      "type" [plain @34:33..34:37]: "object" [plain @34:39..34:45]
                      "title" [plain @35:33..35:38]: "Receipt" [plain @35:40..35:47]
                      "description" [plain @36:33..36:44]: literal [@36:46..39:1]
                        "The mapping of user ID to receipt. The user ID is the\n"
                        "entity who sent this receipt."
                      "x-pattern-format" [plain @39:33..39:49]: "mx-user-id" [double-quoted @39:51..39:63]
                      "properties" [plain @40:33..40:43]: mapping [block]
                        "ts" [plain @41:37..41:39]: mapping [block]
                          "type" [plain @42:41..42:45]: "integer" [plain @42:47..42:54]
                          "format" [plain @43:41..43:47]: "int64" [plain @43:49..43:54]
                          "description" [plain @44:41..44:52]: "The timestamp the receipt was sent at." [plain @44:54..44:92]
                        "thread_id" [plain @45:37..45:46]: mapping [block]
                          "type" [plain @46:41..46:45]: "string" [plain @46:47..46:53]
                          "x-addedInMatrixVersion" [plain @47:41..47:63]: "1.4" [double-quoted @47:65..47:70]
                          "description" [plain @48:41..48:52]: literal [@48:54..53:1]
                            "The [thread root](/client-server-api/#threading)'s\n"
                            "[event ID](/appendices#event-ids) (or `main`) for which\n"
                            "thread this receipt is intended to be under. If not\n"
                            "specified, the read receipt is *unthreaded* (default)."
                "m.read.private" [double-quoted @53:21..53:37]: mapping [block]
                  "type" [plain @54:25..54:29]: "object" [plain @54:31..54:37]
                  "description" [plain @55:25..55:36]: literal [@55:38..59:1]
                    "Similar to `m.read`, the users who have sent `m.read.private`\n"
                    "receipts for this event. Due to the nature of private read\n"
                    "receipts, this should only ever have the current user's ID."
                  "patternProperties" [plain @59:25..59:42]: mapping [block]
                    "^@" [double-quoted @60:29..60:33]: *receiptUserMap [@60:35..60:50]
          "additionalProperties" [plain @61:9..61:29]: "false" [plain @61:31..61:36]
        "type" [plain @62:5..62:9]: mapping [block]
          "type" [plain @63:9..63:13]: "string" [plain @63:15..63:21]
          "enum" [plain @64:9..64:13]: sequence [flow]
            - "m.receipt" [double-quoted @64:16..64:27]
            - "m.receipt.private" [double-quoted @64:29..64:48]
      "required" [plain @65:1..65:9]: sequence [flow]
        - "type" [double-quoted @65:12..65:18]
        - "content" [double-quoted @65:20..65:29]
    "#);
}
