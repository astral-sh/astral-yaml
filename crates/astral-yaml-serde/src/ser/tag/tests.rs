use super::TagSerializer;
use crate::Error;
use serde::Serializer;

fn rejects<T>(result: Result<T, Error>) {
    assert_eq!(result.err().unwrap().message, "expected a YAML tag string");
}

#[test]
fn only_strings_can_supply_tag_names() {
    rejects(TagSerializer.serialize_bool(true));
    rejects(TagSerializer.serialize_i8(1));
    rejects(TagSerializer.serialize_i16(1));
    rejects(TagSerializer.serialize_i32(1));
    rejects(TagSerializer.serialize_i64(1));
    rejects(TagSerializer.serialize_i128(1));
    rejects(TagSerializer.serialize_u8(1));
    rejects(TagSerializer.serialize_u16(1));
    rejects(TagSerializer.serialize_u32(1));
    rejects(TagSerializer.serialize_u64(1));
    rejects(TagSerializer.serialize_u128(1));
    rejects(TagSerializer.serialize_f32(1.0));
    rejects(TagSerializer.serialize_f64(1.0));
    rejects(TagSerializer.serialize_char('!'));
    rejects(TagSerializer.serialize_bytes(b"!Tag"));
    rejects(TagSerializer.serialize_none());
    rejects(TagSerializer.serialize_some("!Tag"));
    rejects(TagSerializer.serialize_unit());
    rejects(TagSerializer.serialize_unit_struct("Tag"));
    rejects(TagSerializer.serialize_unit_variant("Enum", 0, "Tag"));
    rejects(TagSerializer.serialize_newtype_struct("Tag", "!Tag"));
    rejects(TagSerializer.serialize_newtype_variant("Enum", 0, "Tag", "!Tag"));

    rejects(TagSerializer.serialize_seq(Some(1)));
    rejects(TagSerializer.serialize_tuple(1));
    rejects(TagSerializer.serialize_tuple_struct("Tag", 1));
    rejects(TagSerializer.serialize_tuple_variant("Enum", 0, "Tag", 1));
    rejects(TagSerializer.serialize_map(Some(1)));
    rejects(TagSerializer.serialize_struct("Tag", 1));
    rejects(TagSerializer.serialize_struct_variant("Enum", 0, "Tag", 1));
}
