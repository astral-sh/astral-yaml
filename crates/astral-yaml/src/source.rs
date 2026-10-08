use crate::{Error, ParseOptions, Span, Stream, parse};
use std::borrow::Cow;

/// An encoding recognized by YAML 1.2's byte-pattern detection rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Encoding {
    Utf8,
    Utf16Be,
    Utf16Le,
    Utf32Be,
    Utf32Le,
}

/// Decoded input with access to byte ranges in the original encoding.
///
/// UTF-8 input is borrowed. UTF-16 and UTF-32 input is decoded into an owned string.
/// Parsed scalars can borrow from that string. Parser spans refer to [`Self::text`].
/// Use [`Self::original_span`] to convert them to offsets in the original input.
#[derive(Clone, Debug)]
pub struct Source<'a> {
    bytes: &'a [u8],
    text: Cow<'a, str>,
    encoding: Encoding,
}

impl<'a> Source<'a> {
    /// Decodes UTF-8, UTF-16, or UTF-32, with or without a byte order mark.
    ///
    /// # Errors
    /// Returns a decoding error located in the original bytes, or a size-limit error.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Self::with_limit(bytes, ParseOptions::default().max_input_bytes)
    }

    /// Decodes input, bounding both the original and decoded byte lengths.
    ///
    /// # Errors
    /// Returns a decoding error located in the original bytes, or a size-limit error.
    pub fn with_limit(bytes: &'a [u8], max_bytes: usize) -> Result<Self, Error> {
        enum CodeUnit {
            Utf16,
            Utf32,
        }

        if bytes.len() > max_bytes {
            return Err(error(0, 0, "input byte limit exceeded"));
        }

        let encoding = detect(bytes);
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

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Parses the decoded text. Returned spans, including error spans, use UTF-8 offsets.
    ///
    /// # Errors
    /// Returns a syntax or parser resource-limit error.
    pub fn parse(&self) -> Result<Stream<'_>, Error> {
        parse(&self.text)
    }

    /// Converts a decoded-text span into a span of the original bytes.
    ///
    /// For UTF-16/32 this scans the text through the end of the span. Returns `None`
    /// for out-of-bounds offsets or offsets that split a UTF-8 character.
    #[must_use]
    pub fn original_span(&self, span: Span) -> Option<Span> {
        let before = self.text.get(..span.start)?;
        let content = self.text.get(span.start..span.end)?;
        let length = |s: &str| match self.encoding {
            Encoding::Utf8 => s.len(),
            Encoding::Utf16Be | Encoding::Utf16Le => s.encode_utf16().count() * 2,
            Encoding::Utf32Be | Encoding::Utf32Le => s.chars().count() * 4,
        };

        let start = length(before);
        Some(Span {
            start,
            end: start + length(content),
        })
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

fn detect(bytes: &[u8]) -> Encoding {
    match bytes {
        [0, 0, 0xfe, 0xff, ..] | [0, 0, 0, _, ..] => Encoding::Utf32Be,
        [0xff, 0xfe, 0, 0, ..] | [_, 0, 0, 0, ..] => Encoding::Utf32Le,
        [0xfe, 0xff, ..] | [0, _, ..] => Encoding::Utf16Be,
        [0xff, 0xfe, ..] | [_, 0, ..] => Encoding::Utf16Le,
        _ => Encoding::Utf8,
    }
}
