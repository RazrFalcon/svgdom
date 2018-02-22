// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use types::number;

use super::{
    Command,
    Path,
    Segment,
    SegmentData,
};

use {
    WriteBuffer,
    WriteOptions,
    ToStringWithOptions,
};

struct PrevCmd {
    cmd: Command,
    absolute: bool,
    implicit: bool,
}

impl WriteBuffer for Path {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        if self.is_empty() {
            return;
        }

        let mut prev_cmd: Option<PrevCmd> = None;
        let mut prev_coord_has_dot = false;

        for seg in self.iter() {
            let is_written = write_cmd(seg, &mut prev_cmd, opt, buf);
            write_segment(seg.data(), is_written, &mut prev_coord_has_dot, opt, buf);
        }

        if !opt.use_compact_path_notation {
            let len = buf.len();
            buf.truncate(len - 1);
        }
    }
}

fn write_cmd(
    seg: &Segment,
    prev_cmd: &mut Option<PrevCmd>,
    opt: &WriteOptions,
    buf: &mut Vec<u8>
) -> bool {
    let mut print_cmd = true;
    if opt.remove_duplicated_path_commands {
        // check that previous command is the same as current
        if let Some(ref pcmd) = *prev_cmd {
            // MoveTo commands can't be skipped
            if pcmd.cmd != Command::MoveTo {
                if seg.cmd() == pcmd.cmd && seg.absolute == pcmd.absolute {
                    print_cmd = false;
                }
            }
        }
    }

    let mut is_implicit = false;
    if opt.use_implicit_lineto_commands {

        let check_implicit = || {
            if let Some(ref pcmd) = *prev_cmd {
                if seg.absolute != pcmd.absolute {
                    return false;
                }

                if pcmd.implicit {
                    if seg.cmd() == Command::LineTo {
                        return true;
                    }
                } else if    pcmd.cmd  == Command::MoveTo
                          && seg.cmd() == Command::LineTo {
                    // if current segment is LineTo and previous was MoveTo
                    return true;
                }
            }

            false
        };

        if check_implicit() {
            is_implicit = true;
            print_cmd = false;
        }
    }

    *prev_cmd = Some(PrevCmd {
        cmd: seg.cmd(),
        absolute: seg.absolute,
        implicit: is_implicit,
    });

    if !print_cmd {
        // we do not update 'prev_cmd' if we do not wrote it
        return false;
    }

    write_cmd_char(seg, buf);

    if !(seg.cmd() == Command::ClosePath || opt.use_compact_path_notation) {
        buf.push(b' ');
    }

    true
}

pub fn write_cmd_char(seg: &Segment, buf: &mut Vec<u8>) {
    let cmd: u8 = if seg.is_absolute() {
        match seg.cmd() {
            Command::MoveTo => b'M',
            Command::LineTo => b'L',
            Command::HorizontalLineTo => b'H',
            Command::VerticalLineTo => b'V',
            Command::CurveTo => b'C',
            Command::SmoothCurveTo => b'S',
            Command::Quadratic => b'Q',
            Command::SmoothQuadratic => b'T',
            Command::EllipticalArc => b'A',
            Command::ClosePath => b'Z',
        }
    } else {
        match seg.cmd() {
            Command::MoveTo => b'm',
            Command::LineTo => b'l',
            Command::HorizontalLineTo => b'h',
            Command::VerticalLineTo => b'v',
            Command::CurveTo => b'c',
            Command::SmoothCurveTo => b's',
            Command::Quadratic => b'q',
            Command::SmoothQuadratic => b't',
            Command::EllipticalArc => b'a',
            Command::ClosePath => b'z',
        }
    };
    buf.push(cmd);
}

pub fn write_segment(
    data: &SegmentData,
    is_written: bool,
    prev_coord_has_dot: &mut bool,
    opt: &WriteOptions,
    buf: &mut Vec<u8>
) {
    match *data {
          SegmentData::MoveTo { x, y }
        | SegmentData::LineTo { x, y }
        | SegmentData::SmoothQuadratic { x, y } => {
            write_coords(&[x, y], is_written, prev_coord_has_dot, opt, buf);
        }

        SegmentData::HorizontalLineTo { x } => {
            write_coords(&[x], is_written, prev_coord_has_dot, opt, buf);
        }

        SegmentData::VerticalLineTo { y } => {
            write_coords(&[y], is_written, prev_coord_has_dot, opt, buf);
        }

        SegmentData::CurveTo { x1, y1, x2, y2, x, y } => {
            write_coords(&[x1, y1, x2, y2, x, y], is_written,
                         prev_coord_has_dot, opt, buf);
        }

        SegmentData::SmoothCurveTo { x2, y2, x, y } => {
            write_coords(&[x2, y2, x, y], is_written, prev_coord_has_dot, opt, buf);
        }

        SegmentData::Quadratic { x1, y1, x, y } => {
            write_coords(&[x1, y1, x, y], is_written, prev_coord_has_dot, opt, buf);
        }

        SegmentData::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
            write_coords(&[rx, ry, x_axis_rotation], is_written,
                         prev_coord_has_dot, opt, buf);

            if opt.use_compact_path_notation {
                // flags must always have a space before it
                buf.push(b' ');
            }

            write_flag(large_arc, buf);
            if !opt.join_arc_to_flags {
                buf.push(b' ');
            }
            write_flag(sweep, buf);
            if !opt.join_arc_to_flags {
                buf.push(b' ');
            }

            // reset, because flags can't have dots
            *prev_coord_has_dot = false;

            // 'is_explicit_cmd' is always 'true'
            // because it's relevant only for first coordinate of the segment
            write_coords(&[x, y], true, prev_coord_has_dot, opt, buf);
        }
        SegmentData::ClosePath => {
            if !opt.use_compact_path_notation {
                buf.push(b' ');
            }
        }
    }
}

fn write_coords(
    coords: &[f64],
    is_explicit_cmd: bool,
    prev_coord_has_dot: &mut bool,
    opt: &WriteOptions,
    buf: &mut Vec<u8>
) {
    if opt.use_compact_path_notation {
        for (i, num) in coords.iter().enumerate() {
            let start_pos = buf.len() - 1;

            number::write_num(num, opt.remove_leading_zero, buf);

            let c = buf[start_pos + 1];

            let write_space = if !*prev_coord_has_dot && c == b'.' {
                !(i == 0 && is_explicit_cmd)
            } else if i == 0 && is_explicit_cmd {
                false
            } else if (c as char).is_digit(10) {
                true
            } else {
                false
            };

            if write_space {
                buf.insert(start_pos + 1, b' ');
            }

            *prev_coord_has_dot = false;
            for c in buf.iter().skip(start_pos) {
                if *c == b'.' {
                    *prev_coord_has_dot = true;
                    break;
                }
            }
        }
    } else {
        for num in coords.iter() {
            number::write_num(num, opt.remove_leading_zero, buf);
            buf.push(b' ');
        }
    }
}

fn write_flag(flag: bool, buf: &mut Vec<u8>) {
    buf.push(if flag { b'1' } else { b'0' });
}

impl_display!(Path);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use types::path::*;
    use {WriteOptions, ToStringWithOptions};

    #[test]
    fn gen_path_1() {
        let mut path = Path::new();
        path.push(Segment::new_move_to(10.0, 20.0));
        path.push(Segment::new_line_to(10.0, 20.0));
        assert_eq!(path.to_string(), "M 10 20 L 10 20");
    }

    #[test]
    fn gen_path_2() {
        let path = Path::from_str("M 10 20 l 10 20").unwrap();
        assert_eq!(path.to_string(), "M 10 20 l 10 20");
    }

    #[test]
    fn gen_path_3() {
        let path = Path::from_str(
            "M 10 20 L 30 40 H 50 V 60 C 70 80 90 100 110 120 \
             S 130 140 150 160 Q 170 180 190 200 T 210 220 \
             A 50 50 30 1 1 230 240 Z").unwrap();
        assert_eq_text!(path.to_string(),
            "M 10 20 L 30 40 H 50 V 60 C 70 80 90 100 110 120 \
             S 130 140 150 160 Q 170 180 190 200 T 210 220 \
             A 50 50 30 1 1 230 240 Z");
    }

    #[test]
    fn gen_path_4() {
        let path = Path::from_str(
            "m 10 20 l 30 40 h 50 v 60 c 70 80 90 100 110 120 \
             s 130 140 150 160 q 170 180 190 200 t 210 220 \
             a 50 50 30 1 1 230 240 z").unwrap();
        assert_eq_text!(path.to_string(),
            "m 10 20 l 30 40 h 50 v 60 c 70 80 90 100 110 120 \
             s 130 140 150 160 q 170 180 190 200 t 210 220 \
             a 50 50 30 1 1 230 240 z");
    }

    #[test]
    fn gen_path_5() {
        let path = Path::from_str("").unwrap();
        assert_eq_text!(path.to_string(), "");
    }

    macro_rules! test_gen_path_opt {
        ($name:ident, $in_text:expr, $out_text:expr, $flag:ident) => (
            #[test]
            fn $name() {
                let path = Path::from_str($in_text).unwrap();

                let mut opt = WriteOptions::default();
                opt.$flag = true;

                assert_eq_text!(path.to_string_with_opt(&opt), $out_text);
            }
        )
    }

    test_gen_path_opt!(gen_path_6,
        "M 10 20 L 30 40 L 50 60 l 70 80",
        "M 10 20 L 30 40 50 60 l 70 80",
        remove_duplicated_path_commands);

    test_gen_path_opt!(gen_path_7,
        "M 10 20 30 40 50 60",
        "M 10 20 L 30 40 50 60",
        remove_duplicated_path_commands);

    test_gen_path_opt!(gen_path_8,
        "M 10 20 L 30 40",
        "M10 20L30 40",
        use_compact_path_notation);

    test_gen_path_opt!(gen_path_9,
        "M 10 20 V 30 H 40 V 50 H 60 Z",
        "M10 20V30H40V50H60Z",
        use_compact_path_notation);

    #[test]
    fn gen_path_10() {
        let path = Path::from_str("M 10 -20 A 5.5 0.3 -4 1 1 0 -0.1").unwrap();

        let mut opt = WriteOptions::default();
        opt.use_compact_path_notation = true;
        opt.join_arc_to_flags = true;
        opt.remove_leading_zero = true;

        assert_eq_text!(path.to_string_with_opt(&opt), "M10-20A5.5.3-4 110-.1");
    }

    test_gen_path_opt!(gen_path_11,
        "M 10-10 a 1 1 0 1 1 -1 1",
        "M10-10a1 1 0 1 1 -1 1",
        use_compact_path_notation);

    test_gen_path_opt!(gen_path_12,
        "M 10-10 a 1 1 0 1 1 0.1 1",
        "M10-10a1 1 0 1 1 0.1 1",
        use_compact_path_notation);

    test_gen_path_opt!(gen_path_13,
        "M 10 20 L 30 40 L 50 60 H 10",
        "M 10 20 30 40 50 60 H 10",
        use_implicit_lineto_commands);

    // should be ignored, because of different 'absolute' values
    test_gen_path_opt!(gen_path_14,
        "M 10 20 l 30 40 L 50 60",
        "M 10 20 l 30 40 L 50 60",
        use_implicit_lineto_commands);

    test_gen_path_opt!(gen_path_15,
        "M 10 20 L 30 40 l 50 60 L 50 60",
        "M 10 20 30 40 l 50 60 L 50 60",
        use_implicit_lineto_commands);

    test_gen_path_opt!(gen_path_16,
        "M 10 20 L 30 40 l 50 60",
        "M 10 20 30 40 l 50 60",
        use_implicit_lineto_commands);

    test_gen_path_opt!(gen_path_17,
        "M 10 20 L 30 40 L 50 60 M 10 20 L 30 40 L 50 60",
        "M 10 20 30 40 50 60 M 10 20 30 40 50 60",
        use_implicit_lineto_commands);

    #[test]
    fn gen_path_18() {
        let path = Path::from_str("M 10 20 L 30 40 L 50 60 M 10 20 L 30 40 L 50 60").unwrap();

        let mut opt = WriteOptions::default();
        opt.use_implicit_lineto_commands = true;
        opt.remove_duplicated_path_commands = true;

        assert_eq_text!(path.to_string_with_opt(&opt), "M 10 20 30 40 50 60 M 10 20 30 40 50 60");
    }

    #[test]
    fn gen_path_19() {
        let path = Path::from_str("m10 20 A 10 10 0 1 0 0 0 A 2 2 0 1 0 2 0").unwrap();

        let mut opt = WriteOptions::default();
        opt.use_compact_path_notation = true;
        opt.remove_duplicated_path_commands = true;
        opt.remove_leading_zero = true;

        // may generate as 'm10 20A10 10 0 1 0 0 0 2 2 0 1 0  2 0' <- two spaces

        assert_eq_text!(path.to_string_with_opt(&opt), "m10 20A10 10 0 1 0 0 0 2 2 0 1 0 2 0");
    }

    #[test]
    fn gen_path_20() {
        let path = Path::from_str("M 0.1 0.1 L 1 0.1 2 -0.1").unwrap();

        let mut opt = WriteOptions::default();
        opt.use_compact_path_notation = true;
        opt.remove_duplicated_path_commands = true;
        opt.remove_leading_zero = true;

        assert_eq_text!(path.to_string_with_opt(&opt), "M.1.1L1 .1 2-.1");
    }

    test_gen_path_opt!(gen_path_21,
        "M 10 20 M 30 40 M 50 60 L 30 40",
        "M 10 20 M 30 40 M 50 60 L 30 40",
        remove_duplicated_path_commands);
}
