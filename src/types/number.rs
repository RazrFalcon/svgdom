// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Write;
use std::cmp;

use float_cmp::ApproxEqUlps;

use {
    WriteBuffer,
    WriteOptions,
};

/// The trait for comparing f64 numbers.
pub trait FuzzyEq {
    /// Returns `true` if numbers are equal.
    fn fuzzy_eq(&self, other: &f64) -> bool;

    /// Returns `true` if numbers are not equal.
    #[inline]
    fn fuzzy_ne(&self, other: &f64) -> bool {
        !self.fuzzy_eq(other)
    }

    /// Returns `true` if number is zero.
    #[inline]
    fn is_fuzzy_zero(&self) -> bool {
        self.fuzzy_eq(&0.0)
    }
}

impl FuzzyEq for f64 {
    #[inline]
    fn fuzzy_eq(&self, other: &f64) -> bool {
        self.approx_eq_ulps(other, 4)
    }
}

/// The trait for `Ordering` f64 numbers.
pub trait FuzzyOrd {
    /// This method returns an `Ordering` between `self` and `other`.
    fn fuzzy_cmp(&self, other: &f64) -> cmp::Ordering;
}

impl FuzzyOrd for f64 {
    #[inline]
    fn fuzzy_cmp(&self, other: &f64) -> cmp::Ordering {
        if self.fuzzy_eq(other) {
            return cmp::Ordering::Equal;
        } else if self > other {
            return cmp::Ordering::Greater;
        }

        cmp::Ordering::Less
    }
}

pub fn write_num(num: &f64, rm_leading_zero: bool, buf: &mut Vec<u8>) {
    // By default it will round numbers up to 12 digits
    // to prevent writing ugly numbers like 29.999999999999996.
    // It's not 100% correct, but differences are insignificant.

    let v = (num * 1_000_000_000_000.0).round() / 1_000_000_000_000.0;

    let start_pos = buf.len();

    write!(buf, "{}", v).unwrap();

    if rm_leading_zero {
        let mut has_dot = false;
        let mut pos = 0;
        for c in buf.iter().skip(start_pos) {
            if *c == b'.' {
                has_dot = true;
                break;
            }
            pos += 1;
        }

        if has_dot && buf[start_pos + pos - 1] == b'0' {
            if pos == 2 && num.is_sign_negative() {
                // -0.1 -> -.1
                buf.remove(start_pos + 1);
            } else if pos == 1 && num.is_sign_positive() {
                // 0.1 -> .1
                buf.remove(start_pos);
            }
        }
    }
}

impl WriteBuffer for f64 {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        write_num(self, opt.remove_leading_zero, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_number {
        ($name:ident, $num:expr, $rm_zero:expr, $result:expr) => (
            #[test]
            fn $name() {
                let mut v = Vec::new();
                write_num(&$num, $rm_zero, &mut v);
                assert_eq!(String::from_utf8(v).unwrap(), $result);
            }
        )
    }

    test_number!(gen_number_1,  1.0,                 false, "1");
    test_number!(gen_number_2,  0.0,                 false, "0");
    test_number!(gen_number_3,  -0.0,                false, "0");
    test_number!(gen_number_4,  -1.0,                false, "-1");
    test_number!(gen_number_5,  12345678.12345678,   false, "12345678.12345678");
    test_number!(gen_number_6,  -0.1,                true,  "-.1");
    test_number!(gen_number_7,  0.1,                 true,  ".1");
    test_number!(gen_number_8,  1.0,                 true,  "1");
    test_number!(gen_number_9,  -1.0,                true,  "-1");
    test_number!(gen_number_10, 1.5,                 false, "1.5");
    test_number!(gen_number_11, 0.14186,             false, "0.14186");
    test_number!(gen_number_12, 29.999999999999996,  false, "30");
    test_number!(gen_number_13, 0.49999999999999994, false, "0.5");
}
