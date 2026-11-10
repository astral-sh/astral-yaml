//! Write YAML 1.2 scalars and flow collections.

mod flow;

pub use flow::{FlowWrite, LimitedWriter};

use std::fmt::{self, Write};

/// The YAML core-schema spelling of an infinity or NaN.
#[inline]
#[must_use]
pub fn non_finite_float(value: f64) -> Option<&'static str> {
    if value.is_finite() {
        None
    } else if value.is_nan() {
        Some(".nan")
    } else if value.is_sign_negative() {
        Some("-.inf")
    } else {
        Some(".inf")
    }
}

/// Whether a character is outside the [YAML printable character set].
///
/// [YAML printable character set]: https://yaml.org/spec/1.2.2/#51-character-set
#[must_use]
pub fn non_printable(c: char) -> bool {
    (c.is_control() && !matches!(c, '\t' | '\n' | '\r' | '\u{85}'))
        || matches!(c, '\u{fffe}' | '\u{ffff}')
}

/// Whether the emitter requires a double-quoted scalar for a non-printable
/// character or a byte-order mark.
#[must_use]
pub fn requires_double_quotes(c: char) -> bool {
    non_printable(c) || c == '\u{feff}'
}

/// Write a double-quoted scalar, escaping characters that cannot appear literally.
///
/// ```
/// let mut output = String::new();
/// astral_yaml::emit::write_quoted(&mut output, "a\nb").unwrap();
/// assert_eq!(output, r#""a\nb""#);
/// ```
///
/// # Errors
/// Returns the underlying writer's error if it fails.
#[inline]
pub fn write_quoted(output: &mut impl Write, value: &str) -> fmt::Result {
    output.write_char('"')?;
    if value.len() >= 64 {
        quoted_long(output, value)?;
    } else if short_needs_escape(value) {
        write_escaped(output, value)?;
    } else {
        output.write_str(value)?;
    }
    output.write_char('"')
}

#[inline]
fn short_needs_escape(value: &str) -> bool {
    // Check eight bytes at a time without alignment requirements. The last
    // word overlaps its predecessor when the length is not a multiple of eight.
    let bytes = value.as_bytes();
    #[allow(clippy::needless_bitwise_bool)]
    if let Some(last) = bytes.last_chunk::<8>() {
        let mut remaining = bytes;
        let mut escaped = false;
        while let Some((chunk, rest)) = remaining.split_first_chunk::<8>() {
            escaped |= word_needs_escape(u64::from_ne_bytes(*chunk));
            remaining = rest;
        }
        if !remaining.is_empty() {
            escaped |= word_needs_escape(u64::from_ne_bytes(*last));
        }
        escaped
    } else if let (Some(first), Some(last)) = (bytes.first_chunk(), bytes.last_chunk()) {
        let first = u32::from_ne_bytes(*first);
        let last = u32::from_ne_bytes(*last);
        word_needs_escape(u64::from(first) | (u64::from(last) << 32))
    } else {
        value.bytes().fold(false, |escaped, byte| {
            escaped | !(b' '..=b'~').contains(&byte) | (byte == b'"') | (byte == b'\\')
        })
    }
}

#[inline]
fn word_needs_escape(word: u64) -> bool {
    const ONES: u64 = 0x0101_0101_0101_0101;
    const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
    const SPACES: u64 = 0x2020_2020_2020_2020;

    // High bits detect non-ASCII bytes. Adding one detects DEL. Subtracting spaces
    // detects controls. Carries and borrows cannot hide the first offending byte,
    // and printable ASCII causes neither.
    let non_printable = word | word.wrapping_add(ONES) | word.wrapping_sub(SPACES);

    // XOR turns quotes and backslashes into zero bytes. The zero-byte test
    // sets a high bit if either character occurs anywhere in the word.
    let quotes = word ^ u64::from_ne_bytes([b'"'; 8]);
    let slashes = word ^ u64::from_ne_bytes([b'\\'; 8]);
    let escaped_quotes = quotes.wrapping_sub(ONES) & !quotes;
    let escaped_slashes = slashes.wrapping_sub(ONES) & !slashes;

    (non_printable | escaped_quotes | escaped_slashes) & HIGH_BITS != 0
}

#[inline(never)]
fn quoted_long(output: &mut impl Write, value: &str) -> fmt::Result {
    // Quotes, backslashes, and line breaks use memchr below. Other controls
    // and non-ASCII text use the full escape scanner.
    #[allow(clippy::needless_bitwise_bool)]
    let unusual = value.bytes().fold(false, |unusual, byte| {
        unusual | (!(b' '..=b'~').contains(&byte) & (byte != b'\n'))
    });

    if unusual {
        write_escaped(output, value)
    } else {
        let mut start = 0;
        for index in memchr::memchr3_iter(b'"', b'\\', b'\n', value.as_bytes()) {
            output.write_str(&value[start..index])?;
            let escape = match value.as_bytes()[index] {
                b'"' => "\\\"",
                b'\\' => "\\\\",
                _ => "\\n",
            };
            output.write_str(escape)?;
            start = index + 1;
        }
        output.write_str(&value[start..])
    }
}

#[cold]
#[inline(never)]
fn write_escaped(output: &mut impl Write, mut value: &str) -> fmt::Result {
    const fn possible_escape(byte: usize) -> bool {
        (byte < 0x20) | matches!(byte, 0x22 | 0x5c | 0x7f | 0xc2 | 0xe2 | 0xef)
    }
    const ESCAPE: [bool; 256] = {
        let mut escape = [false; 256];
        let mut byte = 0;

        while byte < escape.len() {
            escape[byte] = possible_escape(byte);
            byte += 1;
        }

        escape
    };

    // These possible escapes are ASCII bytes and selected UTF-8 leading bytes,
    // so every candidate index is a character boundary.
    if value.len() >= 64
        && !value
            .bytes()
            .fold(false, |any, byte| any | possible_escape(usize::from(byte)))
    {
        return output.write_str(value);
    }
    // Keep printable Unicode characters in the same uninterrupted run.
    let mut plain = value;
    let mut hex = *b"\\u0000";
    while let Some(index) = value.bytes().position(|byte| ESCAPE[usize::from(byte)]) {
        let rest = &value[index..];
        let mut chars = rest.chars();
        let c = chars.next().expect("matched a candidate byte");
        value = chars.as_str();
        let escape = match c {
            '"' => "\\\"",
            '\\' => "\\\\",
            '\n' => "\\n",
            '\r' => "\\r",
            '\t' => "\\t",
            c if requires_double_quotes(c) || matches!(c, '\u{85}' | '\u{2028}' | '\u{2029}') => {
                // Every code point escaped here fits in a four-digit YAML escape.
                for (digit, shift) in hex[2..].iter_mut().zip([12, 8, 4, 0]) {
                    *digit = b"0123456789ABCDEF"[(c as usize >> shift) & 0xf];
                }
                std::str::from_utf8(&hex).expect("ASCII hex digits")
            }
            _ => continue,
        };
        let prefix_len = plain.len() - rest.len();
        if prefix_len != 0 {
            output.write_str(&plain[..prefix_len])?;
        }
        output.write_str(escape)?;
        plain = value;
    }
    output.write_str(plain)
}
