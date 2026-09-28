use crate::Error;
use serde::{Serialize, ser};

#[cfg(test)]
mod tests;

/// Extracts the runtime tag string from the private Tagged tuple protocol.
pub(super) struct TagSerializer;

macro_rules! reject {
    ($($method:ident($($argument:ident: $ty:ty),*));* $(;)?) => {
        $(
            fn $method(self, $($argument: $ty),*) -> Result<Self::Ok, Error> {
                Err(Error::new("expected a YAML tag string"))
            }
        )*
    };
}

impl ser::Serializer for TagSerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    fn serialize_str(self, tag: &str) -> Result<String, Error> {
        if tag == "!" {
            return Ok(tag.to_owned());
        }

        // Reuse the parser's URI and tag-handle validation. Comparing the
        // expanded tag also rejects strings that inject additional YAML.
        let spelling = format!("!<{tag}>");
        let stream = astral_yaml::parse(&spelling).map_err(|_| Error::new("invalid YAML tag"))?;
        let node = stream.node(stream.documents()[0].root).unwrap();
        if stream.documents().len() != 1
            || node.tag.as_ref().map(|property| property.value.as_ref()) != Some(tag)
        {
            return Err(Error::new("invalid YAML tag"));
        }

        if tag.strip_prefix('!').is_some_and(|name| {
            name.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        }) {
            Ok(tag.to_owned())
        } else {
            Ok(spelling)
        }
    }

    reject! {
        serialize_bool(_value: bool);
        serialize_i8(_value: i8);
        serialize_i16(_value: i16);
        serialize_i32(_value: i32);
        serialize_i64(_value: i64);
        serialize_i128(_value: i128);
        serialize_u8(_value: u8);
        serialize_u16(_value: u16);
        serialize_u32(_value: u32);
        serialize_u64(_value: u64);
        serialize_u128(_value: u128);
        serialize_f32(_value: f32);
        serialize_f64(_value: f64);
        serialize_char(_value: char);
        serialize_bytes(_value: &[u8]);
        serialize_none();
        serialize_unit();
        serialize_unit_struct(_name: &'static str);
        serialize_unit_variant(_name: &'static str, _index: u32, _variant: &'static str);
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _: &T) -> Result<String, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<String, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<String, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Error> {
        Err(Error::new("expected a YAML tag string"))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(Error::new("expected a YAML tag string"))
    }
}
