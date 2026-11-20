use super::{FlowWrite, Output, PyResult};

pub(super) enum Decimal {
    Integer(i64),
    Millionths(i64),
}

impl Decimal {
    /// Recognize floats whose shortest spelling has at most six fractional digits.
    pub(super) fn from_float(value: f64) -> Option<Self> {
        // Debug uses fixed notation starting at 1e-4. Below 1e9 each float's
        // rounding interval is narrower than a millionth, and the scaled
        // integer fits exactly in f64. If a millionth rounds to this float,
        // its spelling with trailing zeros removed is the shortest one.
        if !(1e-4..1e9).contains(&value.abs()) {
            return None;
        }
        let scaled = (value * 1e6).round();
        if (scaled / 1e6).to_bits() != value.to_bits() {
            return None;
        }
        #[allow(clippy::cast_possible_truncation)] // Rounded and bounded by 1e15.
        Some(Self::Millionths(scaled as i64))
    }

    pub(super) fn write(self, output: &mut Output) -> PyResult<()> {
        let (value, mut places) = match self {
            Self::Integer(value) => (value, 0),
            Self::Millionths(value) => (value, 6),
        };
        let mut digits = value.unsigned_abs();
        // Retain one fractional digit for integral floats.
        while places > 1 && digits.is_multiple_of(10) {
            digits /= 10;
            places -= 1;
        }

        // 19 decimal digits, a sign, and a decimal point.
        let mut buffer = [0; 21];
        let mut start = buffer.len();
        let point = buffer.len() - places;
        loop {
            if places > 0 && start == point {
                start -= 1;
                buffer[start] = b'.';
            }
            start -= 1;
            buffer[start] = b'0' + (digits % 10) as u8;
            digits /= 10;
            if digits == 0 && start < point {
                break;
            }
        }
        if value < 0 {
            start -= 1;
            buffer[start] = b'-';
        }
        output.push_str(std::str::from_utf8(&buffer[start..]).expect("ASCII decimal"))
    }
}
