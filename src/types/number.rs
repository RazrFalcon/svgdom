// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Write;

use {WriteOptions, WriteBuffer};

// TODO: add method to compare with specific precision

pub fn write_num(num: f64, rm_leading_zero: bool, buf: &mut Vec<u8>) {
    // We always round a number to 8 digits.
    // f64 can handle up to 15 numbers, but we still round to 8 top.
    let v = (num * 100000000.0f64).round() / 100000000.0f64;

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
            if pos == 2 && v.is_sign_negative() {
                // -0.1 -> -.1
                buf.remove(start_pos + 1);
            } else if pos == 1 && v.is_sign_positive() {
                // 0.1 -> .1
                buf.remove(start_pos);
            }
        }
    }
}

impl WriteBuffer for f64 {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        write_num(*self, opt.remove_leading_zero, buf);
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
                write_num($num, $rm_zero, &mut v);
                assert_eq!(String::from_utf8(v).unwrap(), $result);
            }
        )
    }

    test_number!(gen_number_1, 1.0,                 false, "1");
    test_number!(gen_number_2, 0.0,                 false, "0");
    test_number!(gen_number_3, -0.0,                false, "0");
    test_number!(gen_number_4, -1.0,                false, "-1");
    test_number!(gen_number_5, 12345678.12345678,   false, "12345678.12345678");
    test_number!(gen_number_6, -0.1,                true,  "-.1");
    test_number!(gen_number_7, 0.1,                 true,  ".1");
    test_number!(gen_number_8, 1.0,                 true,  "1");
    test_number!(gen_number_9, -1.0,                true,  "-1");
    test_number!(gen_number_10, 1.5,                false, "1.5");
    test_number!(gen_number_11, 0.14186,            false, "0.14186");
    test_number!(gen_number_12, 0.4621799999999894, false, "0.46218");
    test_number!(gen_number_13, 0.0338000000000136, false, "0.0338");
}
