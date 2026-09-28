#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_serde::{Tagged, from_str, to_string};
use libfuzzer_sys::fuzz_target;
use serde::{Serialize, Serializer};
use yaml_serde::Value;

struct ValueRef<'a>(&'a Value);

impl Serialize for ValueRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Sequence(items) => serializer.collect_seq(items.iter().map(ValueRef)),
            Value::Mapping(entries) => serializer.collect_map(
                entries
                    .iter()
                    .map(|(key, value)| (ValueRef(key), ValueRef(value))),
            ),
            // yaml_serde's TaggedValue uses its own private serialization
            // convention. Use our wrapper to emit tags with our serializer.
            Value::Tagged(tagged) => Tagged {
                tag: tagged.tag.to_string(),
                value: ValueRef(&tagged.value),
            }
            .serialize(serializer),
            value => value.serialize(serializer),
        }
    }
}

fn has_incompatible_tag(value: &Value) -> bool {
    match value {
        Value::Tagged(tagged) => {
            tagged.tag.to_string().contains(['%', '#']) || has_incompatible_tag(&tagged.value)
        }
        Value::Sequence(items) => items.iter().any(has_incompatible_tag),
        Value::Mapping(entries) => entries
            .iter()
            .any(|(key, value)| has_incompatible_tag(key) || has_incompatible_tag(value)),
        _ => false,
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let quoted = to_string(source).unwrap();
    assert_eq!(from_str::<String>(&quoted).unwrap(), source);
    assert_eq!(yaml_serde::from_str::<String>(&quoted).unwrap(), source);

    match from_str::<Value>(source) {
        Ok(value) => {
            if let Ok(encoded) = to_string(&ValueRef(&value)) {
                assert_eq!(from_str::<Value>(&encoded).unwrap(), value);
                // LibYAML decodes tag URI escapes and rejects `#` in tags.
                if !has_incompatible_tag(&value) {
                    assert_eq!(yaml_serde::from_str::<Value>(&encoded).unwrap(), value);
                }
            }
        }
        Err(error) => {
            assert!(
                error.span.is_some_and(|span| span.slice(source).is_some()),
                "{error:?}"
            );
        }
    }

    if let Ok(value) = from_str::<Tagged<Value>>(source) {
        let serializable = Tagged {
            tag: value.tag.clone(),
            value: ValueRef(&value.value),
        };

        if let Ok(encoded) = to_string(&serializable) {
            assert_eq!(from_str::<Tagged<Value>>(&encoded).unwrap(), value);
        }
    }
});
