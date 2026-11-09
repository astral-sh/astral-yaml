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
    const ESCAPE: [bool; 256] = {
        let mut escape = [false; 256];
        let mut byte = 0;

        while byte < escape.len() {
            escape[byte] = byte < 0x20 || matches!(byte, 0x22 | 0x5c | 0x7f | 0xc2 | 0xe2 | 0xef);
            byte += 1;
        }

        escape
    };

    // Copy runs of text until a possible escape. The non-ASCII characters escaped
    // below start with C2, E2, or EF. Stopping at those leading bytes or at ASCII
    // preserves UTF-8 character boundaries.
    loop {
        let index = value
            .bytes()
            .position(|byte| ESCAPE[usize::from(byte)])
            .unwrap_or(value.len());
        let (prefix, rest) = value.split_at(index);
        output.write_str(prefix)?;

        let mut chars = rest.chars();
        let Some(c) = chars.next() else { return Ok(()) };
        let escaped = match c {
            '"' => output.write_str("\\\""),
            '\\' => output.write_str("\\\\"),
            '\n' => output.write_str("\\n"),
            '\r' => output.write_str("\\r"),
            '\t' => output.write_str("\\t"),
            c if requires_double_quotes(c) || matches!(c, '\u{85}' | '\u{2028}' | '\u{2029}') => {
                write!(output, "\\u{:04X}", u32::from(c))
            }
            c => output.write_char(c),
        };
        escaped?;
        value = chars.as_str();
    }
}
