//! YAML 1.2 core-schema scalar classification (specification section 10.3.2).

/// The implicit core-schema type of a plain scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarType {
    Null,
    Bool,
    Int,
    Float,
    String,
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

fn classify_number(value: &str) -> ScalarType {
    if matches!(value, ".nan" | ".NaN" | ".NAN") {
        return ScalarType::Float;
    }

    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);

    // After an optional sign, numeric scalars begin with a digit or a dot.
    // Other prefixes need no further numeric checks.
    if !matches!(unsigned.as_bytes().first(), Some(b'0'..=b'9' | b'.')) {
        return ScalarType::String;
    }

    if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
        return ScalarType::Float;
    }

    if digits(unsigned, 10)
        || value.strip_prefix("0o").is_some_and(|s| digits(s, 8))
        || value.strip_prefix("0x").is_some_and(|s| digits(s, 16))
    {
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
