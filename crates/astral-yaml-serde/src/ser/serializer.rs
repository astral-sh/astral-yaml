#[cfg(feature = "editable")]
use super::INSERTED_BYTE_LIMIT;
use super::{keys, tag};
use crate::with::yaml_tag;
use crate::{Error, tagged};
#[cfg(feature = "editable")]
use astral_yaml::Span;
#[cfg(feature = "editable")]
use astral_yaml::emit::LimitedWriter;
use astral_yaml::emit::{FlowWrite, non_finite_float};
use astral_yaml::schema::CoreType;
use serde::Serialize;
use serde::ser::{self, SerializeMap, SerializeSeq};
use std::fmt;

type Result<T> = std::result::Result<T, Error>;

const MAX_DEPTH: usize = 63;
const EXPLICIT_KEY: &str = "? ";

/// Serialize `value` as a YAML 1.2 document in flow style.
///
/// Strings are quoted to preserve their type. Unit enum variants use their name,
/// and variants with data use a mapping with the variant name as its key.
///
/// The default reader has separate limits on input bytes, syntax elements, and
/// deserialization work. Large serialized documents can exceed those limits.
///
/// # Errors
/// Propagates errors from `Serialize`. Also returns an error if a tag conflicts
/// with its value, a mapping repeats a key, keys and values are not paired,
/// or a resource limit is exceeded.
#[inline]
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    Serializer::serialize_document(value, String::new())
}

#[cfg(feature = "editable")]
pub(crate) fn to_string_with_limit<T: Serialize + ?Sized>(
    value: &T,
    limit: usize,
    span: Span,
) -> Result<String> {
    let on_limit = || Error::at(INSERTED_BYTE_LIMIT, span);
    // The fragment includes the newline after the serialized value.
    let limit = limit.checked_sub(1).ok_or_else(on_limit)?;
    Serializer::serialize_document(value, LimitedWriter::new(limit, on_limit))
}

#[cfg(feature = "editable")]
pub(crate) fn quote_with_limit(value: &str, limit: usize, span: Span) -> Result<String> {
    let mut output = LimitedWriter::new(limit, || Error::at(INSERTED_BYTE_LIMIT, span));
    output.quote(value)?;
    output.finish()
}

struct Serializer<O> {
    output: O,
    depth: usize,
    pending_tag: Option<tag::Constraint>,
    yaml_tag: bool,
    keys: keys::Keys,
}

#[derive(Clone, Copy)]
enum Collection {
    Sequence,
    Mapping,
}

impl<O: FlowWrite<Error>> Serializer<O> {
    #[inline]
    fn serialize_document<T: Serialize + ?Sized>(value: &T, output: O) -> Result<String> {
        let mut serializer = Self {
            output,
            depth: 0,
            pending_tag: None,
            yaml_tag: false,
            keys: keys::Keys::default(),
        };

        value.serialize(&mut serializer)?;
        let mut output = serializer.output.finish()?;
        output.push('\n');
        Ok(output)
    }

    fn enter(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            return Err(Error::new("serialization nesting limit exceeded"));
        }

        if self.depth == 0 {
            self.output.reserve(1024);
        }

        self.depth += 1;
        Ok(())
    }

    #[inline]
    fn compound(&mut self, kind: Collection) -> Result<Compound<'_, O>> {
        // Nested collections without a core tag only need a depth increment.
        let tag = self.take_core_tag();
        if self.depth == 0 || self.depth >= MAX_DEPTH || tag.is_some() {
            self.enter_compound(kind, tag)?;
        } else {
            self.depth += 1;
        }

        let (opening, state) = match kind {
            Collection::Sequence => ("[", CompoundState::Sequence),
            Collection::Mapping => ("{", CompoundState::MapKey),
        };
        self.output.push_str(opening)?;
        let key_scope = self.keys.scope();
        Ok(Compound {
            serializer: self,
            first: true,
            key_scope,
            state,
            variant: false,
        })
    }

    #[cold]
    fn enter_compound(&mut self, kind: Collection, tag: Option<CoreType>) -> Result<()> {
        Self::check_collection_tag(tag, kind)?;
        self.enter()
    }

    fn variant(&mut self, name: &str, kind: Collection) -> Result<Compound<'_, O>> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(name)?;
            self.output.push_str(" ")?;
            return self.compound(kind);
        }

        self.enter()?;
        Self::check_collection_tag(self.take_core_tag(), Collection::Mapping)?;
        self.output.push_str("{")?;
        let start = self.output.as_ref().len();
        self.output.quote(name)?;
        self.output.finish_implicit_key(start)?;

        let mut compound = self.compound(kind)?;
        compound.variant = true;
        Ok(compound)
    }

    fn number(&mut self, value: impl fmt::Display) -> Result<()> {
        let start = self.output.as_ref().len();
        self.output.display(value)?;
        self.check_written_scalar_tag(start)
    }

    fn float(&mut self, value: impl fmt::Display + Into<f64> + Copy) -> Result<()> {
        let start = self.output.as_ref().len();
        if let Some(spelling) = non_finite_float(value.into()) {
            self.output.push_str(spelling)?;
        } else {
            self.output.display(value)?;
            if !self.output.as_ref()[start..].contains(['.', 'e', 'E']) {
                self.output.push_str(".0")?;
            }
        }

        self.check_written_scalar_tag(start)
    }

    /// Consume the current node's tag state and return any core type constraint.
    #[inline]
    fn take_core_tag(&mut self) -> Option<CoreType> {
        self.yaml_tag = false;
        match self.pending_tag.take() {
            Some(tag::Constraint::Core(kind)) => Some(kind),
            Some(tag::Constraint::Any) | None => None,
        }
    }

    #[inline]
    fn check_scalar_tag(&mut self, value: &str) -> Result<()> {
        if let Some(tag) = self.take_core_tag() {
            Self::check_tagged_scalar(tag, value)?;
        }

        Ok(())
    }

    #[inline]
    fn check_written_scalar_tag(&mut self, start: usize) -> Result<()> {
        if let Some(tag) = self.take_core_tag() {
            Self::check_tagged_scalar(tag, &self.output.as_ref()[start..])?;
        }

        Ok(())
    }

    #[cold]
    fn check_tagged_scalar(tag: CoreType, value: &str) -> Result<()> {
        if !tag.matches_scalar(value) {
            return Err(Error::new("serialized value does not match its YAML tag"));
        }

        Ok(())
    }

    fn check_collection_tag(tag: Option<CoreType>, kind: Collection) -> Result<()> {
        if let Some(tag) = tag {
            let actual = match kind {
                Collection::Sequence => CoreType::Sequence,
                Collection::Mapping => CoreType::Mapping,
            };
            if tag != actual {
                return Err(Error::new("serialized value does not match its YAML tag"));
            }
        }

        Ok(())
    }

    fn require_untagged(&self) -> Result<()> {
        if self.pending_tag.is_some() {
            return Err(Error::new("a YAML node cannot have multiple tags"));
        }

        Ok(())
    }

    fn write_tag(&mut self, tag: tag::Tag) -> Result<()> {
        self.pending_tag = Some(tag.write(&mut self.output)?);
        Ok(())
    }

    fn enum_tag(&mut self, variant: &str) -> Result<()> {
        if variant.is_empty() {
            return Err(Error::new("an enum tag requires a nonempty variant name"));
        }

        self.require_untagged()?;
        self.output.check(variant.len().saturating_add(1))?;
        let tag = tag::Tag::parse(&format!("!{variant}"), &mut self.output)?;
        self.write_tag(tag)
    }
}

macro_rules! integer {
    ($($method:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method(self, value: $ty) -> Result<()> {
                self.number(value)
            }
        )*
    };
}

impl<'ser, O: FlowWrite<Error>> ser::Serializer for &'ser mut Serializer<O> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'ser, O>;
    type SerializeTuple = Compound<'ser, O>;
    type SerializeTupleStruct = Compound<'ser, O>;
    type SerializeTupleVariant = Compound<'ser, O>;
    type SerializeMap = Compound<'ser, O>;
    type SerializeStruct = Compound<'ser, O>;
    type SerializeStructVariant = Compound<'ser, O>;

    integer! {
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
    }

    fn serialize_bool(self, value: bool) -> Result<()> {
        let value = if value { "true" } else { "false" };
        self.check_scalar_tag(value)?;
        self.output.push_str(value)
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        self.float(value)
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        self.float(value)
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_str(value.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        if let Some(tag) = self.take_core_tag() {
            self.output.check(value.len().saturating_add(2))?;
            Serializer::<O>::check_tagged_scalar(tag, value)?;
        }
        self.output.quote(value)
    }

    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<()> {
        let value = self.output.collect(value, 2)?;
        self.serialize_str(&value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let mut seq = self.compound(Collection::Sequence)?;

        for byte in value {
            seq.separator()?;
            seq.serializer.number(byte)?;
        }

        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<()> {
        self.enter()?;
        value.serialize(&mut *self)?;
        self.depth -= 1;
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        self.check_scalar_tag("null")?;
        self.output.push_str("null")
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        if std::mem::take(&mut self.yaml_tag) {
            // A node immediately after the explicit-key indicator will be followed
            // by a colon. Separate it from the bare tag.
            let is_key = self.output.as_ref().ends_with(EXPLICIT_KEY);
            self.enum_tag(variant)?;
            self.pending_tag = None;
            if is_key {
                self.output.push_str(" ")?;
            }
            return Ok(());
        }

        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<()> {
        let previous = self.yaml_tag;
        self.yaml_tag |= name == yaml_tag::NAME;
        let result = self.serialize_some(value);
        self.yaml_tag = previous;
        result
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(variant)?;
            self.output.push_str(" ")?;
            return self.serialize_some(value);
        }

        let mut map = self.compound(Collection::Mapping)?;
        map.serialize_entry(variant, value)?;
        SerializeMap::end(map)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'ser, O>> {
        self.compound(Collection::Sequence)
    }

    fn serialize_tuple(self, _: usize) -> Result<Compound<'ser, O>> {
        self.compound(Collection::Sequence)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Compound<'ser, O>> {
        if name == tagged::NAME {
            if len != 2 {
                return Err(Error::new("a tagged value requires a tag and payload"));
            }

            self.require_untagged()?;
            self.enter()?;
            self.yaml_tag = false;
            let key_scope = self.keys.scope();
            return Ok(Compound {
                serializer: self,
                first: true,
                key_scope,
                state: CompoundState::Tag,
                variant: false,
            });
        }

        self.compound(Collection::Sequence)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'ser, O>> {
        self.variant(variant, Collection::Sequence)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'ser, O>> {
        self.compound(Collection::Mapping)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'ser, O>> {
        self.compound(Collection::Mapping)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'ser, O>> {
        self.variant(variant, Collection::Mapping)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CompoundState {
    Sequence,
    MapKey,
    MapValue,
    Tag,
    TaggedValue,
    TaggedEnd,
    Failed,
}

impl CompoundState {
    fn closing_delimiter(self) -> Result<Option<&'static str>> {
        match self {
            Self::Sequence => Ok(Some("]")),
            Self::MapKey => Ok(Some("}")),
            Self::TaggedEnd => Ok(None),
            Self::MapValue => Err(Error::new("mapping key has no value")),
            Self::Tag | Self::TaggedValue => {
                Err(Error::new("a tagged value requires a tag and payload"))
            }
            Self::Failed => Err(Error::new("collection serialization already failed")),
        }
    }
}

struct Compound<'ser, O> {
    serializer: &'ser mut Serializer<O>,
    first: bool,
    key_scope: keys::Scope,
    state: CompoundState,
    variant: bool,
}

impl<O: FlowWrite<Error>> Compound<'_, O> {
    #[inline]
    fn separator(&mut self) -> Result<()> {
        self.serializer.output.separator(&mut self.first)
    }

    /// Leave the collection failed until the current field has been written in full.
    fn take_state(&mut self) -> Result<CompoundState> {
        match std::mem::replace(&mut self.state, CompoundState::Failed) {
            CompoundState::Failed => Err(Error::new("collection serialization already failed")),
            state => Ok(state),
        }
    }

    fn finish(self) -> Result<()> {
        if self.state == CompoundState::MapKey {
            self.serializer
                .keys
                .finish(self.serializer.output.as_ref(), self.key_scope)?;
        }

        if let Some(closing) = self.state.closing_delimiter()? {
            self.serializer.output.push_str(closing)?;
        }
        self.serializer.depth -= 1;

        if self.variant {
            self.serializer.output.push_str("}")?;
            self.serializer.depth -= 1;
        }

        Ok(())
    }
}

impl<O: FlowWrite<Error>> SerializeSeq for Compound<'_, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        let state = self.take_state()?;
        match state {
            CompoundState::Tag => {
                let tag = value.serialize(tag::TagSerializer(&mut self.serializer.output))?;
                self.serializer.write_tag(tag)?;
                self.serializer.output.push_str(" ")?;
                self.state = CompoundState::TaggedValue;
                return Ok(());
            }
            CompoundState::TaggedValue => {
                value.serialize(&mut *self.serializer)?;
                self.state = CompoundState::TaggedEnd;
                return Ok(());
            }
            CompoundState::TaggedEnd => {
                return Err(Error::new("a tagged value has more than two fields"));
            }
            _ => {}
        }

        self.separator()?;
        value.serialize(&mut *self.serializer)?;
        self.state = state;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.finish()
    }
}

macro_rules! sequence {
    ($trait:ident, $method:ident) => {
        impl<O: FlowWrite<Error>> ser::$trait for Compound<'_, O> {
            type Ok = ();
            type Error = Error;

            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
                SerializeSeq::serialize_element(self, value)
            }

            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}

sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);

impl<O: FlowWrite<Error>> SerializeMap for Compound<'_, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<()> {
        if self.take_state()? != CompoundState::MapKey {
            return Err(Error::new("mapping key has no value"));
        }

        self.separator()?;
        self.serializer.output.push_str(EXPLICIT_KEY)?;
        let start = self.serializer.output.as_ref().len();
        key.serialize(&mut *self.serializer)?;
        let output = self.serializer.output.as_ref();
        let signature = self.serializer.keys.insert(output, start..output.len())?;
        self.key_scope.insert(signature);
        self.serializer.output.push_str(": ")?;
        self.state = CompoundState::MapValue;
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        if self.take_state()? != CompoundState::MapValue {
            return Err(Error::new("mapping value has no key"));
        }

        value.serialize(&mut *self.serializer)?;
        self.state = CompoundState::MapKey;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.finish()
    }
}

macro_rules! structure {
    ($trait:ident) => {
        impl<O: FlowWrite<Error>> ser::$trait for Compound<'_, O> {
            type Ok = ();
            type Error = Error;

            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<()> {
                self.serialize_entry(key, value)
            }

            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}

structure!(SerializeStruct);
structure!(SerializeStructVariant);

#[cfg(all(test, feature = "editable"))]
mod tests {
    use super::{INSERTED_BYTE_LIMIT, Span, quote_with_limit, to_string, to_string_with_limit};
    use crate::Tagged;
    use serde::{Serialize, Serializer};
    use std::cell::Cell;
    use std::fmt;
    use test_case::test_case;

    #[test_case("\t\u{7}\u{feff}é"; "short escaped string")]
    #[test_case(&format!("{}\\", "a".repeat(64)); "long ASCII with an escape")]
    #[test_case(&"é\u{7}".repeat(30); "long non-ASCII with escapes")]
    #[test_case(&42_u8; "integer")]
    #[test_case(&42.0_f64; "float")]
    #[test_case(&[1, 2]; "sequence")]
    #[test_case(&Tagged { tag: "!Local".to_owned(), value: 1 }; "local tag")]
    #[test_case(&Tagged { tag: "tag:example.com,2026:value".to_owned(), value: 1 }; "verbatim tag")]
    fn limit_counts_serialized_bytes<T: Serialize + ?Sized>(value: &T) {
        let expected = to_string(value).unwrap();
        let span = Span::new(0, 0).unwrap();
        assert_eq!(
            to_string_with_limit(value, expected.len(), span).unwrap(),
            expected
        );
        let error = to_string_with_limit(value, expected.len() - 1, span).unwrap_err();
        assert_eq!(error.message, INSERTED_BYTE_LIMIT);
    }

    #[test]
    fn over_budget_escaped_key_reports_the_edit_span() {
        let span = Span::new(4, 8).unwrap();
        let error = quote_with_limit("\n", 3, span).unwrap_err();
        assert_eq!(error, crate::Error::at(INSERTED_BYTE_LIMIT, span));
    }

    struct Formatted<T>(T);

    impl<T: fmt::Display> Serialize for Formatted<T> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(&self.0)
        }
    }

    struct GeneratedText<'a> {
        writes: &'a Cell<usize>,
        swallow_error: bool,
    }

    impl fmt::Display for GeneratedText<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for _ in 0..100 {
                self.writes.set(self.writes.get() + 1);
                if f.write_str("text").is_err() {
                    return if self.swallow_error {
                        Ok(())
                    } else {
                        Err(fmt::Error)
                    };
                }
            }
            Ok(())
        }
    }

    #[test_case(false)]
    #[test_case(true)]
    fn collected_text_stops_at_the_budget(swallow_error: bool) {
        let writes = Cell::new(0);
        let value = Formatted(GeneratedText {
            writes: &writes,
            swallow_error,
        });
        let span = Span::new(0, 0).unwrap();
        let error = to_string_with_limit(&value, 9, span).unwrap_err();
        assert_eq!(error.message, INSERTED_BYTE_LIMIT);
        assert_eq!(writes.get(), 2);
    }

    // https://yaml.org/spec/1.2.2/#57-escaped-characters
    #[test]
    fn collected_text_uses_normal_escaping() {
        let value = Formatted("é\n");
        let span = Span::new(0, 0).unwrap();
        assert_eq!(to_string(&value).unwrap(), "\"é\\n\"\n");
        assert_eq!(to_string_with_limit(&value, 7, span).unwrap(), "\"é\\n\"\n");
    }

    #[test]
    fn collected_text_can_use_the_full_budget() {
        let writes = Cell::new(0);
        let value = Formatted(GeneratedText {
            writes: &writes,
            swallow_error: false,
        });
        let span = Span::new(0, 0).unwrap();
        let expected = format!("\"{}\"\n", "text".repeat(100));
        assert_eq!(
            to_string_with_limit(&value, expected.len(), span).unwrap(),
            expected
        );
        assert_eq!(writes.get(), 100);
    }

    #[test]
    fn display_errors_propagate_with_and_without_a_limit() {
        struct Broken;

        impl fmt::Display for Broken {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Err(fmt::Error)
            }
        }

        let span = Span::new(0, 0).unwrap();
        let error = to_string(&Formatted(Broken)).unwrap_err();
        assert_eq!(error.message, "display formatting failed");
        let limited = to_string_with_limit(&Formatted(Broken), 10, span).unwrap_err();
        assert_eq!(limited, error);
    }
}
