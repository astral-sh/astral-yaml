#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-functional-tests-res-comments-unencrypted-comments-32c0402b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "lorem" [plain @2:1..2:6]: "ENC[AES256_GCM,data:qVz4paM=,iv:0oGsaw71i3wZKmlyDl8uDhQT9XLvJt3oIyx514X44K8=,tag:acbMS613StWo1IVnKK+5uQ==,type:str]" [plain @2:8..2:123]
      "dolor" [plain @4:1..4:6]: "ENC[AES256_GCM,data:21fI,iv:01LXdHZYwLTeyUB1YWIAM6KF8cPPVsw/RuQO+Ab4pgM=,tag:o1xnCIIoccWzdWxB2kZYKg==,type:str]" [plain @4:8..4:119]
      "sops" [plain @5:1..5:5]: mapping [block]
        "kms" [plain @6:5..6:8]: sequence [flow]
        "gcp_kms" [plain @7:5..7:12]: sequence [flow]
        "azure_kv" [plain @8:5..8:13]: sequence [flow]
        "lastmodified" [plain @9:5..9:17]: "2019-08-29T22:42:03Z" [single-quoted @9:19..9:41]
        "mac" [plain @10:5..10:8]: "ENC[AES256_GCM,data:xKkcsqrAHxyqwgv+IVqx52AmrJdC607Dc/Ughna2e2UnnHteXTw7LGt4d0sSlw8LgjaXpa+T6lQ0MgnMPjgTEa20lbtVtauCDdRCnR7Z/Vdk7t7uLl94+STD7C1H6obnOe4fG6c2cUfDNHoeABLetti2ZBZOSZDkWQeCucLys/s=,iv:k9ODJLNYsBedQKMHcgn0KUXPlunOq5jDFH9BeJOyYRE=,tag:d+Ga71V+gijiWLFZ7BhQgg==,type:str]" [plain @10:10..10:289]
        "pgp" [plain @11:5..11:8]: sequence [block]
          - mapping [block]
            "created_at" [plain @12:9..12:19]: "2019-08-29T22:42:03Z" [single-quoted @12:21..12:43]
            "enc" [plain @13:9..13:12]: literal [@13:14..26:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "wcBMAyUpShfNkFB/AQgAMgUtep3vurVXoOI1h4Ovr3YbYz+gRHAPGMAUdXQdX7az\n"
              "fmh7Eq+6Eye2wpnAaogJ2RtIoYO1F/jkoQO74mgaLXNu/gtr8b6Ejc61sQSyjnjg\n"
              "N9I50+Bh075TqzToZTo7gOwOlltMjA/UVGD0z+gPHP1MpcVpUvm7C0Ol5L0Co37O\n"
              "UvLrQJjw1x6ktAawWokw910iX9usXiTj87fYvaqutNKRBfh1LI0Os2H8C9xpJu7q\n"
              "I60NcqF7JQbAaaumRroQpF2K7RI1nt+qTshPqWzDLfjzlrCVPnJlxludqvHUKzR3\n"
              "TnJdb4Dsx3o/XEQkZxpl7RZa9SZZuKg3EejTxOj0ZtLgAeRLzLl0gSeMGvkntASq\n"
              "NQkM4e1p4AjgJOEqrOCw4h0xtdrg9+UczwVpI4rnibRqJZtEhFHRaupX6leAEi1l\n"
              "FMikSjpGSOB55PVMt6+e4ruokoulnq32VRTi07691uEeowA=\n"
              "=ft04\n"
              "-----END PGP MESSAGE-----"
            "fp" [plain @26:9..26:11]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4" [plain @26:13..26:53]
        "unencrypted_suffix" [plain @27:5..27:23]: "_unencrypted" [plain @27:25..27:37]
        "version" [plain @28:5..28:12]: "3.3.1" [plain @28:14..28:19]
    "#);
}
