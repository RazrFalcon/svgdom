use dtoa;

use {WriteOptions, WriteBuffer};

// TODO: add method to compare with specific precision

static POW_VEC: &'static [f64] = &[
    0.0,
    10.0,
    100.0,
    1000.0,
    10000.0,
    100000.0,
    1000000.0,
    10000000.0,
    100000000.0,
];

static THRESHOLD_VEC: &'static [f64] = &[
    0.0,
    0.01,
    0.001,
    0.0001,
    0.00001,
    0.000001,
    0.0000001,
    0.00000001,
    0.000000001,
];

pub fn write_num(num: f64, precision: u8, rm_leading_zero: bool, buf: &mut Vec<u8>) {
    debug_assert!(precision <= 8 && precision != 0);

    let multiplier = POW_VEC[precision as usize];
    let tmp_value = (num * multiplier).round().abs() as u64;

    if tmp_value == 0 {
        buf.push(b'0');
        return;
    }

    let mut new_value = (tmp_value as f64) / multiplier;

    let threshold = THRESHOLD_VEC[precision as usize];
    if (new_value - new_value.floor()) / new_value < threshold {
        new_value = new_value.floor();
    }

    new_value = new_value * num.signum();

    let start_pos = buf.len();

    dtoa::write(buf, new_value).unwrap();

    // dtoa is always adds '.0', so we have to remove it
    // yes, it's ugly, but fast
    if buf[buf.len() - 1] == b'0' {
        if buf[buf.len() - 2] == b'.' {
            let new_len = buf.len() - 2;
            buf.truncate(new_len);
        }
    }

    if rm_leading_zero {
        let mut has_dot = false;
        let mut pos = 0;
        for i in start_pos..buf.len() {
            if buf[i] == b'.' {
                has_dot = true;
                break;
            }
            pos += 1;
        }

        if has_dot && buf[start_pos + pos - 1] == b'0' {
            if pos == 2 && new_value.is_sign_negative() {
                // -0.1 -> -.1
                buf.remove(start_pos + 1);
            } else if pos == 1 && new_value.is_sign_positive() {
                // 0.1 -> .1
                buf.remove(start_pos);
            }
        }
    }
}

impl WriteBuffer for f64 {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        write_num(*self, opt.numbers.precision_coordinates, opt.numbers.remove_leading_zero, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_number {
        ($name:ident, $num:expr, $precision:expr, $rm_zero:expr, $result:expr) => (
            #[test]
            fn $name() {
                let mut v = Vec::new();
                write_num($num, $precision, $rm_zero, &mut v);
                assert_eq!(String::from_utf8(v).unwrap(), $result);
            }
        )
    }

    test_number!(gen_number_1,  1.0,                8, false, "1");
    test_number!(gen_number_2,  1.2345678,          4, false, "1.2346");
    test_number!(gen_number_3,  0.0,                8, false, "0");
    test_number!(gen_number_4,  -0.0,               8, false, "0");
    test_number!(gen_number_5,  -1.0,               8, false, "-1");
    test_number!(gen_number_6,  1.2345678,          2, false, "1.23");
    test_number!(gen_number_7,  1.3333333,          4, false, "1.3333");
    test_number!(gen_number_8,  0.0000001,          4, false, "0");
    test_number!(gen_number_9,  1.0000001,          4, false, "1");
    test_number!(gen_number_10, 0.12555,            2, false, "0.13");
    test_number!(gen_number_11, 0.15,               1, false, "0.2");
    test_number!(gen_number_12, 0.125,              2, false, "0.13");
    test_number!(gen_number_13, 0.14,               1, false, "0.1");
    test_number!(gen_number_14, 12345678.12345678,  8, false, "12345678.12345678");
    test_number!(gen_number_15, -0.1,               8, true,  "-.1");
    test_number!(gen_number_16, 0.1,                8, true,  ".1");
    test_number!(gen_number_17, 1.0,                8, true,  "1");
    test_number!(gen_number_18, -1.0,               8, true,  "-1");
    test_number!(gen_number_19, 80.000005,          6, false, "80");
    test_number!(gen_number_20, 32.000001,          6, false, "32");
    test_number!(gen_number_21, 1.5,                6, false, "1.5");
}
