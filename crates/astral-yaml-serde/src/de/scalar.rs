use crate::Error;
use astral_yaml::Span;
use astral_yaml::schema::{self, ScalarType};
use std::num::ParseIntError;

type Result<T> = std::result::Result<T, Error>;

/// Scalar conversions shared by the document and owned-value deserializers.
#[derive(Clone, Copy)]
pub(crate) struct ResolvedScalar<'a> {
    text: &'a str,
    kind: ScalarType,
    span: Option<Span>,
}

impl<'a> ResolvedScalar<'a> {
    pub(super) fn new(text: &'a str, kind: ScalarType, span: Option<Span>) -> Self {
        Self { text, kind, span }
    }

    pub(crate) fn application(text: &'a str) -> Self {
        Self::new(text, schema::classify(text), None)
    }

    pub(crate) fn is_null(self) -> bool {
        self.kind == ScalarType::Null
    }

    pub(crate) fn unit(self) -> Result<()> {
        if !self.is_null() {
            return Err(Error::at("expected null", self.span));
        }
        Ok(())
    }

    pub(crate) fn boolean(self) -> Result<bool> {
        if self.kind != ScalarType::Bool {
            return Err(Error::at("expected a boolean", self.span));
        }
        Ok(matches!(self.text, "true" | "True" | "TRUE"))
    }

    pub(super) fn integer_parts(self) -> Result<(&'a str, u32, Option<Span>)> {
        if self.kind != ScalarType::Int {
            return Err(Error::at("expected an integer", self.span));
        }
        let (digits, radix) = schema::radix_digits(self.text).unwrap_or((self.text, 10));
        Ok((digits, radix, self.span))
    }

    pub(crate) fn integer<T>(
        self,
        parse: impl FnOnce(&str, u32) -> std::result::Result<T, ParseIntError>,
    ) -> Result<T> {
        let (value, radix, span) = self.integer_parts()?;
        let value = value
            .strip_prefix('-')
            .filter(|digits| digits.bytes().all(|digit| digit == b'0'))
            .unwrap_or(value);
        parse(value, radix).map_err(|_| Error::at("integer out of range", span))
    }

    fn radix_integer_value(self) -> Result<Option<u128>> {
        if self.kind != ScalarType::Int {
            return Ok(None);
        }
        let Some((value, radix)) = schema::radix_digits(self.text) else {
            return Ok(None);
        };
        u128::from_str_radix(value, radix)
            .map(Some)
            .map_err(|_| Error::at("integer out of range", self.span))
    }

    fn wide_radix_float(self) -> Result<f64> {
        let (digits, radix, span) = self.integer_parts()?;
        let digit_bits = if radix == 16 { 4 } else { 3 };
        let mut significand = 0_u64;
        let mut significant_bits = 0;
        let mut discarded_bits = 0;
        let mut guard = false;
        let mut sticky = false;

        for digit in digits.bytes() {
            let digit = char::from(digit)
                .to_digit(radix)
                .expect("classified integer has valid digits");

            for position in (0..digit_bits).rev() {
                let bit = (digit >> position) & 1;
                if significant_bits == 0 && bit == 0 {
                    continue;
                }

                if significant_bits < f64::MANTISSA_DIGITS {
                    significand = (significand << 1) | u64::from(bit);
                    significant_bits += 1;
                } else {
                    if discarded_bits == 0 {
                        guard = bit != 0;
                    } else {
                        sticky |= bit != 0;
                    }

                    discarded_bits += 1;
                    let mantissa_bits = i32::try_from(f64::MANTISSA_DIGITS)
                        .expect("f64 mantissa width fits in i32");
                    if discarded_bits > f64::MAX_EXP - mantissa_bits {
                        return Err(Error::at("float out of range", span));
                    }
                }
            }
        }

        // Round to nearest, ties to even, before scaling by a power of two.
        if guard && (sticky || significand & 1 != 0) {
            significand += 1;
        }

        #[allow(clippy::cast_precision_loss)]
        let value = (significand as f64) * 2_f64.powi(discarded_bits);
        if !value.is_finite() {
            return Err(Error::at("float out of range", span));
        }

        Ok(value)
    }

    pub(crate) fn float64(self) -> Result<f64> {
        if !matches!(self.kind, ScalarType::Float | ScalarType::Int) {
            return Err(Error::at("expected a number", self.span));
        }

        if let Some(value) = schema::special_float(self.text) {
            return Ok(value);
        }

        match self.radix_integer_value() {
            Ok(Some(value)) => {
                // YAML integer-to-float conversion rounds when the mantissa cannot hold every bit.
                #[allow(clippy::cast_precision_loss)]
                return Ok(value as f64);
            }
            Err(_) => return self.wide_radix_float(),
            Ok(None) => {}
        }

        let value: f64 = self
            .text
            .parse()
            .map_err(|_| Error::at("invalid float", self.span))?;
        if !value.is_finite() {
            return Err(Error::at("float out of range", self.span));
        }

        Ok(value)
    }

    pub(crate) fn float32(self) -> Result<f32> {
        // Convert decimal text and radix integers directly to f32 to avoid
        // double rounding through f64. Rust cannot parse YAML's .inf/.nan.
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let (value, finite_input) = if let Some(integer) = self.radix_integer_value()? {
            (integer as f32, true)
        } else {
            let wide = self.float64()?;
            (
                self.text.parse::<f32>().unwrap_or(wide as f32),
                wide.is_finite(),
            )
        };

        if finite_input && !value.is_finite() {
            return Err(Error::at("float out of range", self.span));
        }

        Ok(value)
    }
}
