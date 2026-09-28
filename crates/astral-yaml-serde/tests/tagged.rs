use astral_yaml_dev::Value;
use astral_yaml_serde::{Spanned, Tagged, from_str, to_string};
use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::SerializeTupleStruct};
use std::{collections::BTreeMap, fmt::Debug};

fn roundtrip<T>(value: &Tagged<T>)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let yaml = to_string(value).unwrap();
    assert_eq!(&from_str::<Tagged<T>>(&yaml).unwrap(), value, "{yaml}");
}

#[test]
fn owned_and_borrowed_payloads() {
    let mut value = from_str::<Tagged<String>>("!Text hello").unwrap();
    assert_eq!(value.tag, "!Text");
    assert_eq!(*value, "hello");
    value.push_str(" world");
    roundtrip(&value);
    assert_eq!(value.into_inner(), "hello world");

    let source = "!Text hello";
    let value = from_str::<Tagged<&str>>(source).unwrap();
    assert!(std::ptr::eq(value.value.as_ptr(), source[6..].as_ptr()));
    assert_eq!(value.tag, "!Text");

    for source in ["hello", "[hello]", "{Text: hello}", "null"] {
        assert_eq!(
            from_str::<Tagged<serde::de::IgnoredAny>>(source)
                .unwrap_err()
                .message,
            "expected an explicit YAML tag"
        );
    }

    assert!(from_str::<Tagged<u8>>("!Number 256").is_err());
    assert!(from_str::<Tagged<Tagged<String>>>("!Text hello").is_err());
    assert!(from_str::<Tagged<Tagged<String>>>("!!str hello").is_err());

    let error = serde_json::from_str::<Tagged<String>>(r#"["!Text", "hello"]"#).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("an explicitly tagged YAML value")
    );
}

// https://yaml.org/spec/1.2.2/#691-node-tags
// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn expanded_tags_and_core_types() {
    for (source, tag, text) in [
        ("!Text hello", "!Text", "hello"),
        ("!<!Text> hello", "!Text", "hello"),
        (
            "!<tag:example.com,2026:text> hello",
            "tag:example.com,2026:text",
            "hello",
        ),
        (
            "%TAG !e! tag:example.com,2026:\n--- !e!text hello",
            "tag:example.com,2026:text",
            "hello",
        ),
        (
            "!<tag:example.com,2026:a%20b> hello",
            "tag:example.com,2026:a%20b",
            "hello",
        ),
        ("!!str null", "tag:yaml.org,2002:str", "null"),
        ("! null", "!", "null"),
    ] {
        let value = from_str::<Tagged<String>>(source).unwrap();
        assert_eq!(value.tag, tag);
        assert_eq!(value.value, text);
        roundtrip(&value);
    }

    let value = from_str::<Tagged<u32>>("!!int '42'").unwrap();
    assert_eq!(value.value, 42);
    roundtrip(&value);
    roundtrip(&from_str::<Tagged<bool>>("!!bool 'true'").unwrap());
    roundtrip(&from_str::<Tagged<f64>>("!!float '1.5'").unwrap());
    roundtrip(&from_str::<Tagged<()>>("!!null null").unwrap());
    roundtrip(&from_str::<Tagged<()>>("!Empty").unwrap());
    roundtrip(&from_str::<Tagged<Option<String>>>("!Maybe null").unwrap());
    roundtrip(&from_str::<Tagged<Vec<u32>>>("!!seq [1, 2]").unwrap());
    roundtrip(&from_str::<Tagged<BTreeMap<String, u32>>>("!!map {a: 1}").unwrap());
    assert_eq!(
        from_str::<Tagged<bool>>("!!bool yes").unwrap_err().message,
        "scalar does not match its tag"
    );
    assert_eq!(
        from_str::<Tagged<String>>("!!seq text")
            .unwrap_err()
            .message,
        "unsupported scalar tag"
    );

    for source in [
        "!!bool yes",
        "!!int text",
        "!!seq {}",
        "!!map []",
        "!!str []",
        "!!seq text",
        "!!map text",
    ] {
        assert!(
            from_str::<Tagged<serde::de::IgnoredAny>>(source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn collections_enums_and_nested_tags() {
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Config {
        text: Tagged<String>,
        items: Vec<Tagged<u32>>,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    enum Mode {
        Ready,
        Text(String),
    }

    let source = "!Config {text: !Text hello, items: [!Count 1, !Count 2]}";
    let value = from_str::<Tagged<Config>>(source).unwrap();
    assert_eq!(value.tag, "!Config");
    assert_eq!(value.text.value, "hello");
    assert_eq!(value.items[1].value, 2);
    roundtrip(&value);

    roundtrip(&from_str::<Tagged<Mode>>("!Envelope Ready").unwrap());
    roundtrip(&from_str::<Tagged<Mode>>("!Envelope {Text: hello}").unwrap());
    roundtrip(&from_str::<Tagged<Vec<Mode>>>("!Modes [Ready, {Text: hello}]").unwrap());
    roundtrip(&from_str::<Tagged<(u8, bool)>>("!Pair [1, true]").unwrap());
    roundtrip(&from_str::<Tagged<Vec<Tagged<String>>>>("!Texts [!Text hello]").unwrap());
    assert_eq!(
        from_str::<Option<Tagged<()>>>("!Empty")
            .unwrap()
            .unwrap()
            .tag,
        "!Empty"
    );
}

#[test]
fn generic_yaml_values_reject_local_tags() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Mixed {
        Text(String),

        #[serde(untagged)]
        Number(u32),
    }

    for source in [
        "!Text hello",
        "!Count 1",
        "!Empty",
        "!Items [!Text hello, !Number 42]",
        "!Config {key: !Text hello}",
        "[&x !Text hello, *x]",
    ] {
        assert!(from_str::<Value>(source).is_err(), "{source}");
        assert!(from_str::<serde_json::Value>(source).is_err());
    }

    let error = from_str::<Vec<Mixed>>("[!Text hello, 42]").unwrap_err();
    assert_eq!(
        error.message,
        "non-core YAML tags require Tagged<T> or with::yaml_tag"
    );
    assert_eq!(
        error.span.unwrap().slice("[!Text hello, 42]"),
        Some("!Text hello")
    );
    assert_eq!(
        from_str::<Vec<Mixed>>("[{Text: hello}, 42]").unwrap(),
        [Mixed::Text("hello".into()), Mixed::Number(42)]
    );
}

// https://yaml.org/spec/1.2.2/#56-miscellaneous-characters
#[test]
fn tag_uri_characters_retain_their_spelling() {
    for source in ["!%12", "!#"] {
        let tagged = from_str::<Tagged<()>>(source).unwrap();
        assert_eq!(tagged.tag, source);

        let encoded = to_string(&tagged).unwrap();
        assert_eq!(from_str::<Tagged<()>>(&encoded).unwrap(), tagged);
    }
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test]
fn tags_in_mapping_keys() {
    let source = "{!Text one: 1, !Other one: 2}";
    let value = from_str::<BTreeMap<Tagged<String>, u8>>(source).unwrap();
    assert_eq!(value.len(), 2);
    assert_eq!(
        from_str::<BTreeMap<Tagged<String>, u8>>(&to_string(&value).unwrap()).unwrap(),
        value
    );

    for source in [
        "{!Text one: 1, one: 2}",
        "{!Items [1]: a, !Other [1]: b}",
        "{!Config {a: 1}: a, !Other {a: 1}: b}",
    ] {
        assert!(from_str::<Value>(source).is_err(), "{source}");
    }

    let text_keys =
        from_str::<BTreeMap<Tagged<String>, String>>("{!Count 1: a, !Count 01: b}").unwrap();
    assert_eq!(text_keys.len(), 2);

    for source in [
        "{!Text one: 1, !Text one: 2}",
        "{&key !Text one: 1, *key : 2}",
        "{!Items [1]: a, !Items [1]: b}",
        "{!Config {a: 1, b: 2}: a, !Config {b: 2, a: 1}: b}",
    ] {
        assert_eq!(
            from_str::<Value>(source).unwrap_err().message,
            "duplicate mapping key",
            "{source}"
        );
    }
}

#[test]
fn spans_comments_and_aliases() {
    let source = "# greeting\n!Text hello # inline\n";
    let value = from_str::<Spanned<'_, Tagged<String>>>(source).unwrap();
    assert_eq!(value.tag, "!Text");
    assert_eq!(value.value.value, "hello");
    assert_eq!(value.span.slice(source), Some("!Text hello"));
    assert_eq!(value.comments.len(), 2);
    assert_eq!(
        from_str::<Tagged<String>>(&to_string(&value).unwrap()).unwrap(),
        value.value
    );

    let inner = from_str::<Tagged<Spanned<'_, String>>>(source).unwrap();
    assert_eq!(inner.value.span, value.span);
    assert_eq!(inner.value.comments, value.comments);

    let source = "[&text !Text hello, *text]";
    let values = from_str::<Vec<Spanned<'_, Tagged<&str>>>>(source).unwrap();
    assert_eq!(values[0].value, values[1].value);
    assert_eq!(values[1].span.slice(source), Some("*text"));
}

// https://yaml.org/spec/1.2.2/#69-node-properties
#[test]
fn tag_serialization_rejects_invalid_tags_and_stacked_tags() {
    for tag in [
        "",
        "Text",
        "!bad%",
        "!bad%GG",
        "!bad tag",
        "!bad\n---\n",
        "!x> null #",
    ] {
        let value = Tagged {
            tag: tag.to_owned(),
            value: "hello",
        };
        assert_eq!(
            to_string(&value).unwrap_err().message,
            "invalid YAML tag",
            "{tag}"
        );
    }

    let value = Tagged {
        tag: "!Outer".into(),
        value: Some(Tagged {
            tag: "!Inner".into(),
            value: 42,
        }),
    };
    assert_eq!(
        to_string(&value).unwrap_err().message,
        "a YAML node cannot have multiple tags"
    );
}

#[test]
fn recursive_tagged_values_require_opt_in_at_every_node() {
    for source in ["&self !Sequence [*self]", "&self !Mapping {next: *self}"] {
        assert!(from_str::<Value>(source).is_err());
        assert!(from_str::<serde::de::IgnoredAny>(source).is_err());
        assert!(from_str::<Tagged<serde::de::IgnoredAny>>(source).is_err());
    }
}

struct TagProtocol<'a, T> {
    name: &'static str,
    tag: &'a T,
    declared: usize,
    fields: usize,
}

impl<T: Serialize> Serialize for TagProtocol<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_tuple_struct(self.name, self.declared)?;
        if self.fields > 0 {
            record.serialize_field(self.tag)?;
        }

        for _ in 1..self.fields {
            record.serialize_field(self.tag)?;
        }

        record.end()
    }
}

#[test]
fn tag_serialization_validates_the_private_protocol() {
    let tag = "!Text".to_owned();

    for (declared, fields) in [(0, 0), (3, 2), (2, 0), (2, 1), (2, 3)] {
        assert!(
            to_string(&TagProtocol {
                name: "$astral_yaml::Tagged",
                tag: &tag,
                declared,
                fields
            })
            .is_err()
        );
    }

    let record = TagProtocol {
        name: "$astral_yaml::Tagged",
        tag: &42,
        declared: 2,
        fields: 2,
    };
    assert_eq!(
        to_string(&record).unwrap_err().message,
        "expected a YAML tag string"
    );

    let record = TagProtocol {
        name: "OrdinaryTuple",
        tag: &tag,
        declared: 2,
        fields: 2,
    };
    assert_eq!(to_string(&record).unwrap(), "[\"!Text\", \"!Text\"]\n");
}
