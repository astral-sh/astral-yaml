use crate::{Error, ParseOptions, Span, Stream};
use std::borrow::Cow;

/// An encoding recognized by YAML 1.2's byte-pattern detection rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Encoding {
    /// UTF-8, also the default for empty input.
    Utf8,
    /// UTF-16 with the high byte of each code unit first.
    Utf16Be,
    /// UTF-16 with the low byte of each code unit first.
    Utf16Le,
    /// UTF-32 with the most significant byte first.
    Utf32Be,
    /// UTF-32 with the least significant byte first.
    Utf32Le,
}

impl Encoding {
    fn detect(bytes: &[u8]) -> Self {
        match bytes {
            [0, 0, 0xfe, 0xff, ..] | [0, 0, 0, _, ..] => Self::Utf32Be,
            [0xff, 0xfe, 0, 0, ..] | [_, 0, 0, 0, ..] => Self::Utf32Le,
            [0xfe, 0xff, ..] | [0, _, ..] => Self::Utf16Be,
            [0xff, 0xfe, ..] | [_, 0, ..] => Self::Utf16Le,
            _ => Self::Utf8,
        }
    }

    fn encoded_len(self, text: &str) -> usize {
        match self {
            Self::Utf8 => text.len(),
            Self::Utf16Be | Self::Utf16Le => text.encode_utf16().count() * 2,
            Self::Utf32Be | Self::Utf32Le => text.chars().count() * 4,
        }
    }
}

/// YAML input decoded to UTF-8.
///
/// UTF-8 input is borrowed. UTF-16 and UTF-32 input is decoded into an owned
/// string, which parsed scalars can borrow. Parser spans refer to [`Self::text`].
/// Use [`Self::original_span`] to map them back to the encoded input.
#[derive(Clone, Debug)]
pub struct Source<'bytes> {
    bytes: &'bytes [u8],
    text: Cow<'bytes, str>,
    encoding: Encoding,
}

/// Decodes the input with the same byte limit as [`Source::from_bytes`].
impl<'input> TryFrom<&'input [u8]> for Source<'input> {
    type Error = Error;

    fn try_from(bytes: &'input [u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

impl<'bytes> Source<'bytes> {
    /// Decode UTF-8, UTF-16, or UTF-32 input, with or without a byte order mark.
    /// Without a byte order mark, the first character must be ASCII.
    ///
    /// # Errors
    /// Returns an error if the input is malformed or exceeds the default size limit.
    /// Decoding errors use offsets in the original bytes.
    pub fn from_bytes(bytes: &'bytes [u8]) -> Result<Self, Error> {
        Self::with_limit(bytes, ParseOptions::default().max_input_bytes)
    }

    /// Decode `bytes`, limiting both the encoded and decoded lengths to `max_bytes`.
    ///
    /// # Errors
    /// Returns an error if the input is malformed or either length exceeds `max_bytes`.
    /// Decoding errors use offsets in the original bytes.
    pub fn with_limit(bytes: &'bytes [u8], max_bytes: usize) -> Result<Self, Error> {
        enum CodeUnit {
            Utf16,
            Utf32,
        }

        if bytes.len() > max_bytes {
            return Err(error(0, 0, "input byte limit exceeded"));
        }

        let encoding = Encoding::detect(bytes);
        let unit = match encoding {
            Encoding::Utf16Be | Encoding::Utf16Le => CodeUnit::Utf16,
            Encoding::Utf32Be | Encoding::Utf32Le => CodeUnit::Utf32,
            Encoding::Utf8 => {
                let text = std::str::from_utf8(bytes).map_err(|e| {
                    error(
                        e.valid_up_to(),
                        e.valid_up_to() + e.error_len().unwrap_or(bytes.len() - e.valid_up_to()),
                        "invalid UTF-8",
                    )
                })?;

                if let Some(first) = text.chars().next() {
                    check_initial_character(first, first.len_utf8())?;
                }

                return Ok(Self {
                    bytes,
                    text: Cow::Borrowed(text),
                    encoding,
                });
            }
        };

        let mut text = String::with_capacity(bytes.len());
        let mut offset = 0;

        while offset < bytes.len() {
            let start = offset;
            let c = match unit {
                CodeUnit::Utf16 => {
                    let first = word(bytes, offset, encoding)?;
                    offset += 2;
                    let value = if (0xd800..=0xdbff).contains(&first) {
                        let low = word(bytes, offset, encoding)?;
                        if !(0xdc00..=0xdfff).contains(&low) {
                            return Err(error(start, offset + 2, "invalid UTF-16 surrogate pair"));
                        }
                        offset += 2;
                        0x10000 + (u32::from(first - 0xd800) << 10) + u32::from(low - 0xdc00)
                    } else {
                        u32::from(first)
                    };

                    char::from_u32(value)
                        .ok_or_else(|| error(start, offset, "unpaired UTF-16 surrogate"))?
                }
                CodeUnit::Utf32 => {
                    let Some(chunk) = bytes.get(offset..offset + 4) else {
                        return Err(error(offset, bytes.len(), "truncated UTF-32 code point"));
                    };

                    let array = [chunk[0], chunk[1], chunk[2], chunk[3]];
                    let value = if encoding == Encoding::Utf32Be {
                        u32::from_be_bytes(array)
                    } else {
                        u32::from_le_bytes(array)
                    };
                    offset += 4;
                    char::from_u32(value)
                        .ok_or_else(|| error(start, offset, "invalid UTF-32 code point"))?
                }
            };

            if start == 0 {
                check_initial_character(c, offset)?;
            }

            if text.len() + c.len_utf8() > max_bytes {
                return Err(error(start, offset, "decoded byte limit exceeded"));
            }

            text.push(c);
        }

        Ok(Self {
            bytes,
            text: Cow::Owned(text),
            encoding,
        })
    }

    /// The UTF-8 text used by the parser, including any leading byte order mark.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The original bytes before decoding, including any byte order mark.
    #[must_use]
    pub fn bytes(&self) -> &'bytes [u8] {
        self.bytes
    }

    /// The encoding detected from the original byte pattern.
    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Parse the decoded text. Spans, including error spans, use UTF-8 byte offsets.
    ///
    /// # Errors
    /// Returns an error if the YAML is invalid or a parser resource limit is exceeded.
    pub fn parse(&self) -> Result<Stream<'_>, Error> {
        crate::parse(&self.text)
    }

    /// Map a span in the decoded text to the original encoded bytes.
    ///
    /// Returns `None` for out-of-bounds offsets or offsets that split a UTF-8 character.
    /// For UTF-16 and UTF-32, this scans the text through the end of the span.
    #[must_use]
    pub fn original_span(&self, span: Span) -> Option<Span> {
        let before = self.text.get(..span.start)?;
        let content = self.text.get(span.start..span.end)?;

        let start = self.encoding.encoded_len(before);
        Some(Span {
            start,
            end: start + self.encoding.encoded_len(content),
        })
    }
}

fn check_initial_character(first: char, end: usize) -> Result<(), Error> {
    if first.is_ascii() || first == '\u{feff}' {
        Ok(())
    } else {
        Err(error(
            0,
            end,
            "non-ASCII stream start requires a byte order mark",
        ))
    }
}

fn word(bytes: &[u8], offset: usize, encoding: Encoding) -> Result<u16, Error> {
    let Some(chunk) = bytes.get(offset..offset + 2) else {
        return Err(error(offset, bytes.len(), "truncated UTF-16 code unit"));
    };

    let array = [chunk[0], chunk[1]];
    Ok(if encoding == Encoding::Utf16Be {
        u16::from_be_bytes(array)
    } else {
        u16::from_le_bytes(array)
    })
}

fn error(start: usize, end: usize, message: &'static str) -> Error {
    Error {
        span: Span { start, end },
        message,
    }
}
