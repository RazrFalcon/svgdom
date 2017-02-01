// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use types::LengthUnit;
use {WriteOptions, WriteBuffer, WriteToString};

#[cfg(feature = "parsing")]
use FromStream;
#[cfg(feature = "parsing")]
use svgparser::{Stream, Error as ParseError};

/// Representation of the [`<length>`] type.
/// [`<length>`]: https://www.w3.org/TR/SVG/types.html#DataTypeLength
///
/// We use own struct and not one from svgparser, because of traits.
#[derive(Clone,Copy,PartialEq,Debug)]
#[allow(missing_docs)]
pub struct Length {
    pub num: f64,
    pub unit: LengthUnit,
}

impl Length {
    /// Constructs a new length.
    #[inline]
    pub fn new(num: f64, unit: LengthUnit) -> Length {
        Length {
            num: num,
            unit: unit,
        }
    }

    /// Constructs a new length with a zero value.
    ///
    /// Shorthand for: `Length::new(0.0, Unit::None)`.
    #[inline]
    pub fn zero() -> Length {
        Length {
            num: 0.0,
            unit: LengthUnit::None,
        }
    }
}

#[cfg(feature = "parsing")]
impl FromStream for Length {
    type Err = ParseError;

    fn from_stream(mut s: Stream) -> Result<Length, ParseError> {
        let l = try!(s.parse_length());
        Ok(Length::new(l.num, l.unit))
    }
}

impl WriteBuffer for Length {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        self.num.write_buf_opt(opt, buf);

        let t: &[u8] = match self.unit {
            LengthUnit::None => b"",
            LengthUnit::Em => b"em",
            LengthUnit::Ex => b"ex",
            LengthUnit::Px => b"px",
            LengthUnit::In => b"in",
            LengthUnit::Cm => b"cm",
            LengthUnit::Mm => b"mm",
            LengthUnit::Pt => b"pt",
            LengthUnit::Pc => b"pc",
            LengthUnit::Percent => b"%",
        };

        buf.extend_from_slice(t);
    }
}

impl_display!(Length);

#[cfg(test)]
mod tests {
    use super::*;
    use types::LengthUnit;

    macro_rules! test_length {
        ($name:ident, $len:expr, $unit:expr, $result:expr) => (
            #[test]
            fn $name() {
                let l = Length::new($len, $unit);
                assert_eq!(l.to_string(), $result);
            }
        )
    }

    test_length!(gen_length_1,  1.0, LengthUnit::None, "1");
    test_length!(gen_length_2,  1.0, LengthUnit::Em, "1em");
    test_length!(gen_length_3,  1.0, LengthUnit::Ex, "1ex");
    test_length!(gen_length_4,  1.0, LengthUnit::Px, "1px");
    test_length!(gen_length_5,  1.0, LengthUnit::In, "1in");
    test_length!(gen_length_6,  1.0, LengthUnit::Cm, "1cm");
    test_length!(gen_length_7,  1.0, LengthUnit::Mm, "1mm");
    test_length!(gen_length_8,  1.0, LengthUnit::Pt, "1pt");
    test_length!(gen_length_9,  1.0, LengthUnit::Pc, "1pc");
    test_length!(gen_length_10, 1.0, LengthUnit::Percent, "1%");
}
