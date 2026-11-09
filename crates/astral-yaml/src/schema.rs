//! YAML 1.2 core-schema scalar classification (specification section 10.3.2).

/// The implicit core-schema type of a plain scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarType {
    /// A null spelling, including the empty plain scalar and `~`.
    Null,
    /// A core-schema spelling of `true` or `false`.
    Bool,
    /// A decimal, octal, or hexadecimal integer of any magnitude.
    Int,
    /// A floating-point spelling, including infinity and NaN.
    Float,
    /// Text that does not match another core-schema type.
    String,
}

/// A node type defined by the YAML 1.2 core schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreType {
    /// A scalar with a core-schema type.
    Scalar(ScalarType),
    /// An ordered sequence of nodes.
    Sequence,
    /// A mapping of keys to values.
    Mapping,
}

impl CoreType {
    /// Returns whether the decoded scalar text can be presented with this explicit tag.
    #[must_use]
    pub fn matches_scalar(self, value: &str) -> bool {
        match self {
            Self::Scalar(ScalarType::String) => true,
            Self::Scalar(ScalarType::Float) => {
                matches!(classify(value), ScalarType::Float | ScalarType::Int)
                    && radix_digits(value).is_none()
            }
            Self::Scalar(kind) => classify(value) == kind,
            Self::Sequence | Self::Mapping => false,
        }
    }

    /// Looks up an expanded core tag, such as `tag:yaml.org,2002:str`.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "tag:yaml.org,2002:str" => Some(Self::Scalar(ScalarType::String)),
            "tag:yaml.org,2002:null" => Some(Self::Scalar(ScalarType::Null)),
            "tag:yaml.org,2002:bool" => Some(Self::Scalar(ScalarType::Bool)),
            "tag:yaml.org,2002:int" => Some(Self::Scalar(ScalarType::Int)),
            "tag:yaml.org,2002:float" => Some(Self::Scalar(ScalarType::Float)),
            "tag:yaml.org,2002:seq" => Some(Self::Sequence),
            "tag:yaml.org,2002:map" => Some(Self::Mapping),
            _ => None,
        }
    }
}

/// Classifies a plain, untagged scalar without imposing machine numeric limits.
#[must_use]
#[inline]
pub fn classify(value: &str) -> ScalarType {
    match value.as_bytes().first() {
        None => ScalarType::Null,
        Some(b'~' | b'n' | b'N') if matches!(value, "~" | "null" | "Null" | "NULL") => {
            ScalarType::Null
        }
        Some(b't' | b'T' | b'f' | b'F')
            if matches!(
                value,
                "true" | "True" | "TRUE" | "false" | "False" | "FALSE"
            ) =>
        {
            ScalarType::Bool
        }
        Some(b'+' | b'-' | b'.' | b'0'..=b'9') => classify_number(value),
        _ => ScalarType::String,
    }
}

/// Split the digits and base of an integer with a YAML radix prefix.
#[must_use]
pub fn radix_digits(value: &str) -> Option<(&str, u32)> {
    if let Some(digits) = value.strip_prefix("0x") {
        Some((digits, 16))
    } else {
        value.strip_prefix("0o").map(|digits| (digits, 8))
    }
}

/// Resolve the core schema's non-finite floating-point spellings.
#[must_use]
pub fn special_float(value: &str) -> Option<f64> {
    match value {
        ".nan" | ".NaN" | ".NAN" => Some(f64::NAN),
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        _ => None,
    }
}

fn classify_number(value: &str) -> ScalarType {
    if special_float(value).is_some() {
        return ScalarType::Float;
    }

    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);

    // After an optional sign, numeric scalars begin with a digit or a dot.
    // Other prefixes need no further numeric checks.
    if !matches!(unsigned.as_bytes().first(), Some(b'0'..=b'9' | b'.')) {
        return ScalarType::String;
    }

    if digits(unsigned, 10) || radix_digits(value).is_some_and(|(s, radix)| digits(s, radix)) {
        return ScalarType::Int;
    }

    let (mantissa, exponent) = unsigned.find(['e', 'E']).map_or((unsigned, None), |i| {
        (&unsigned[..i], Some(&unsigned[i + 1..]))
    });
    if exponent.is_some_and(|s| !digits(s.strip_prefix(['+', '-']).unwrap_or(s), 10)) {
        return ScalarType::String;
    }

    let valid = mantissa.find('.').map_or_else(
        || digits(mantissa, 10),
        |i| {
            let (before, after) = (&mantissa[..i], &mantissa[i + 1..]);
            (before.is_empty() || digits(before, 10))
                && (after.is_empty() || digits(after, 10))
                && !(before.is_empty() && after.is_empty())
        },
    );
    if valid {
        ScalarType::Float
    } else {
        ScalarType::String
    }
}

fn digits(value: &str, radix: u32) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii() && c.is_digit(radix))
}

#[cfg(test)]
mod tests {
    use super::{ScalarType, classify};

    // https://yaml.org/spec/1.2.2/#1032-tag-resolution
    #[test]
    fn core_schema() {
        for (kind, values) in [
            (ScalarType::Null, &["", "~", "null", "Null", "NULL"][..]),
            (
                ScalarType::Bool,
                &["true", "True", "TRUE", "false", "False", "FALSE"],
            ),
            (
                ScalarType::Int,
                &[
                    "0",
                    "-12",
                    "+1",
                    "012",
                    "0xFF",
                    "0o77",
                    "99999999999999999999999",
                ],
            ),
            (
                ScalarType::Float,
                &[
                    "1.2", "1.", ".5", "-.5", "1e2", "1.E-2", "+.INF", "-.inf", ".NaN",
                ],
            ),
            (
                ScalarType::String,
                &[
                    "+",
                    "-",
                    "+true",
                    "-null",
                    "yes",
                    "on",
                    "No",
                    "2001-01-01",
                    "1:20",
                    "0b11",
                    "1_000",
                    "+0x1",
                    "0o8",
                    "0x",
                    ".",
                    "1e",
                    "1e+",
                    "1e1e2",
                    "1.2.3",
                    "-.nan",
                    "１２",
                    "Truee",
                    "NULLl",
                ],
            ),
        ] {
            for value in values {
                assert_eq!(classify(value), kind, "{value:?}");
            }
        }
    }
}
