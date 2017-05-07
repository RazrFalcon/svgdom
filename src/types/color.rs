// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use {WriteOptions, WriteBuffer, WriteToString};

#[cfg(feature = "parsing")]
use FromStream;
#[cfg(feature = "parsing")]
use svgparser::{TextFrame, Color as ParserColor, Error as ParseError};

/// Representation of the `<color>` type.
#[derive(Clone,Copy,PartialEq,Debug)]
pub struct Color {
    #[allow(missing_docs)]
    pub red: u8,
    #[allow(missing_docs)]
    pub green: u8,
    #[allow(missing_docs)]
    pub blue: u8,
}

impl Color {
    /// Constructs a new color.
    #[inline]
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
        Color { red: red, green: green, blue: blue }
    }
}

#[cfg(feature = "parsing")]
impl FromStream for Color {
    type Err = ParseError;

    fn from_stream(s: TextFrame) -> Result<Color, ParseError> {
        let c = ParserColor::from_frame(s)?;
        Ok(Color::new(c.red, c.green, c.blue))
    }
}

static CHARS: &'static [u8] = b"0123456789abcdef";

#[inline]
fn int2hex(n: u8) -> (u8, u8) {
    (CHARS[(n >> 4) as usize], CHARS[(n & 0xf) as usize])
}

impl WriteBuffer for Color {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        // TODO: next release: rgb() support
        // TODO: next release: color name support

        buf.push(b'#');
        let (r1, r2) = int2hex(self.red);
        let (g1, g2) = int2hex(self.green);
        let (b1, b2) = int2hex(self.blue);

        if opt.trim_hex_colors && r1 == r2 && g1 == g2 && b1 == b2 {
            buf.push(r1);
            buf.push(g1);
            buf.push(b1);
        } else {
            buf.push(r1);
            buf.push(r2);
            buf.push(g1);
            buf.push(g2);
            buf.push(b1);
            buf.push(b2);
        }
    }
}

impl_display!(Color);

#[cfg(test)]
mod tests {
    use super::*;
    use {WriteOptions, WriteBuffer};

    macro_rules! test_color {
        ($name:ident, $c:expr, $trim:expr, $result:expr) => (
            #[test]
            fn $name() {
                let mut opt = WriteOptions::default();
                opt.trim_hex_colors = $trim;
                let mut out = Vec::new();
                $c.write_buf_opt(&opt, &mut out);
                assert_eq!(String::from_utf8(out).unwrap(), $result);
            }
        )
    }

    test_color!(gen_color_1, Color::new(255, 0, 0), false, "#ff0000");
    test_color!(gen_color_2, Color::new(255, 127, 5), false, "#ff7f05");
    test_color!(gen_color_3, Color::new(255, 0, 0), true, "#f00");
    test_color!(gen_color_4, Color::new(255, 127, 5), true, "#ff7f05");
}
