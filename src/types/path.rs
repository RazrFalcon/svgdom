// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains all struct's for manipulating SVG paths data.

use std::fmt;

use {WriteOptions, FromStream, WriteBuffer, WriteToString};

pub use svgparser::path::{Command, Segment, SegmentData};
use svgparser;
use svgparser::Error as ParseError;
use svgparser::Stream;

/// Representation of SVG path data.
#[derive(PartialEq,Clone)]
pub struct Path {
    /// Vector which contain all the segments.
    pub d: Vec<Segment>
}

impl Path {
    /// Constructs a new path.
    pub fn new() -> Path {
        Path { d: Vec::new() }
    }

    // TODO: append Path
}

// TODO: impl iter for Path

/// Construct a new path using build pattern.
pub struct Builder {
    path: Path,
}

// TODO: Does moving expensive?
impl Builder {
    /// Constructs a new builder.
    pub fn new() -> Builder {
        Builder { path: Path::new() }
    }

    // TODO: from existing Path

    /// Appends a new MoveTo segment.
    pub fn move_to(mut self, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_move_to(x, y));
        self
    }

    /// Appends a new ClosePath segment.
    pub fn close_path(mut self) -> Builder {
        self.path.d.push(Segment::new_close_path());
        self
    }

    /// Appends a new LineTo segment.
    pub fn line_to(mut self, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_line_to(x, y));
        self
    }

    /// Appends a new HorizontalLineTo segment.
    pub fn hline_to(mut self, x: f64) -> Builder {
        self.path.d.push(Segment::new_hline_to(x));
        self
    }

    /// Appends a new VerticalLineTo segment.
    pub fn vline_to(mut self, y: f64) -> Builder {
        self.path.d.push(Segment::new_vline_to(y));
        self
    }

    /// Appends a new CurveTo segment.
    pub fn curve_to(mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_curve_to(x1, y1, x2, y2, x, y));
        self
    }

    /// Appends a new SmoothCurveTo segment.
    pub fn smooth_curve_to(mut self, x2: f64, y2: f64, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_smooth_curve_to(x2, y2, x, y));
        self
    }

    /// Appends a new QuadTo segment.
    pub fn quad_to(mut self, x1: f64, y1: f64, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_quad_to(x1, y1, x, y));
        self
    }

    /// Appends a new SmoothQuadTo segment.
    pub fn smooth_quad_to(mut self, x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_smooth_quad_to(x, y));
        self
    }

    /// Appends a new ArcTo segment.
    pub fn arc_to(mut self, rx: f64, ry: f64, x_axis_rotation: f64, large_arc: bool, sweep: bool,
                  x: f64, y: f64) -> Builder {
        self.path.d.push(Segment::new_arc_to(rx, ry, x_axis_rotation, large_arc, sweep, x, y));
        self
    }

    /// Finalizes the build.
    pub fn finalize(self) -> Path {
        self.path
    }
}

impl FromStream for Path {
    type Err = ParseError;
    fn from_stream(s: Stream) -> Result<Path, ParseError> {
        let mut t = svgparser::path::Tokenizer::new(s);
        let mut p = Path::new();

        while let Some(n) = t.next() {
            match n {
                Ok(segment) => p.d.push(segment),
                Err(e) => return Err(e),
            }
        }

        Ok(p)
    }
}

impl WriteBuffer for Path {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        if self.d.is_empty() {
            return;
        }

        let mut prev_cmd: Option<Command> = None;
        let mut prev_coord_has_dot = false;

        for seg in &self.d {
            let is_written = write_cmd(&seg, &mut prev_cmd, opt, buf);
            match *seg.data() {
                SegmentData::MoveTo { x, y } => {
                    write_coords(&[x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::LineTo { x, y } => {
                    write_coords(&[x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::HorizontalLineTo { x } => {
                    write_coords(&[x], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::VerticalLineTo { y } => {
                    write_coords(&[y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::CurveTo { x1, y1, x2, y2, x, y } => {
                    write_coords(&[x1, y1, x2, y2, x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::SmoothCurveTo { x2, y2, x, y } => {
                    write_coords(&[x2, y2, x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::Quadratic { x1, y1, x, y } => {
                    write_coords(&[x1, y1, x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::SmoothQuadratic { x, y } => {
                    write_coords(&[x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                    write_coords(&[rx, ry, x_axis_rotation], is_written, &mut prev_coord_has_dot, opt, buf);

                    if opt.paths.use_compact_notation {
                        // flag must always have space before it
                        buf.push(b' ');
                    }

                    write_flag(large_arc, buf);
                    // if !opt.paths.use_compact_notation {
                        buf.push(b' ');
                    // }
                    write_flag(sweep, buf);
                    // if !opt.paths.use_compact_notation {
                        buf.push(b' ');
                    // }
                    write_coords(&[x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }
                SegmentData::ClosePath => {
                    if !opt.paths.use_compact_notation {
                        buf.push(b' ');
                    }
                }
            }
        }

        if !opt.paths.use_compact_notation {
            let len = buf.len();
            buf.truncate(len - 1);
        }
    }
}

fn write_cmd(seg: &Segment, prev_cmd: &mut Option<Command>,
             opt: &WriteOptions, buf: &mut Vec<u8>) -> bool {

    let mut print_cmd = true;
    // check is previous command is the same as current
    if opt.paths.remove_duplicated_commands {
        match prev_cmd {
            &mut Some(pcmd) => {
                if *seg.cmd() == pcmd {
                    print_cmd = false;
                }
            }
            &mut None => {}
        }

    }

    if !print_cmd {
        return false;
    }

    buf.push(seg.cmd().data());
    *prev_cmd = Some(*seg.cmd());

    if !(seg.cmd().to_absolute().data() == b'Z' || opt.paths.use_compact_notation) {
        buf.push(b' ');
    }

    true
}

fn write_coords(coords: &[f64], is_explicit_cmd: bool, prev_coord_has_dot: &mut bool,
    opt: &WriteOptions, buf: &mut Vec<u8>) {

    if opt.paths.use_compact_notation {
        for (i, num) in coords.iter().enumerate() {
            let start_pos = buf.len() - 1;

            num.write_buf_opt(opt, buf);

            let c = buf[start_pos + 1];

            let write_space;
            if !*prev_coord_has_dot && c == b'.' {
                write_space = true;
            } else if i == 0 && is_explicit_cmd {
                write_space = false;
            } else if (c as char).is_digit(10)  {
                write_space = true;
            } else {
                write_space = false;
            }

            if write_space {
                buf.insert(start_pos + 1, b' ');
            }

            *prev_coord_has_dot = false;
            for j in start_pos..buf.len() {
                if buf[j] == b'.' {
                    *prev_coord_has_dot = true;
                    break;
                }
            }
        }
    } else {
        for num in coords.iter() {
            num.write_buf_opt(opt, buf);
            buf.push(b' ');
        }
    }
}

fn write_flag(flag: bool, buf: &mut Vec<u8>) {
    if flag {
        buf.push(b'1');
    } else {
        buf.push(b'0');
    }
}

impl_display!(Path);

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

// TODO: to global
macro_rules! assert_eq_text {
    ($left:expr , $right:expr) => ({
        let mut rd = Vec::new();
        rd.extend_from_slice($right);
        match (&$left, &rd) {
            (left_val, rd) => {
                if !(*left_val == *rd) {
                    panic!("assertion failed: `(left == right)` \
                           \nleft:  `{}`\nright: `{}`",
                           String::from_utf8_lossy(left_val),
                           String::from_utf8_lossy($right))
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::path;
    use {WriteOptions, FromStream, WriteBuffer};

    #[test]
    fn gen_path_1() {
        let mut path = Path::new();
        path.d.push(path::Segment::new_move_to(10.0, 20.0));
        path.d.push(path::Segment::new_line_to(10.0, 20.0).to_relative());

        let mut buf = Vec::new();
        path.write_buf(&mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "M 10 20 l 10 20");
    }

    #[test]
    fn gen_path_2() {
        let path = Path::from_data(b"M 10 20 l 10 20").unwrap();
        let mut buf = Vec::new();
        path.write_buf(&mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "M 10 20 l 10 20");
    }

    #[test]
    fn gen_path_3() {
        let path = Path::from_data(b"M 10 20 L 30 40 H 50 V 60 C 70 80 90 100 110 120 \
                                     S 130 140 150 160 Q 170 180 190 200 T 210 220 \
                                     A 50 50 30 1 1 230 240 Z").unwrap();
        let mut buf = Vec::new();
        path.write_buf(&mut buf);
        assert_eq_text!(buf, b"M 10 20 L 30 40 H 50 V 60 C 70 80 90 100 110 120 \
                               S 130 140 150 160 Q 170 180 190 200 T 210 220 \
                               A 50 50 30 1 1 230 240 Z");
    }

    #[test]
    fn gen_path_4() {
        let path = Path::from_data(b"m 10 20 l 30 40 h 50 v 60 c 70 80 90 100 110 120 \
                                     s 130 140 150 160 q 170 180 190 200 t 210 220 \
                                     a 50 50 30 1 1 230 240 z").unwrap();
        let mut buf = Vec::new();
        path.write_buf(&mut buf);
        assert_eq_text!(buf, b"m 10 20 l 30 40 h 50 v 60 c 70 80 90 100 110 120 \
                               s 130 140 150 160 q 170 180 190 200 t 210 220 \
                               a 50 50 30 1 1 230 240 z");
    }

    #[test]
    fn gen_path_5() {
        let path = Path::from_data(b"").unwrap();
        let mut buf = Vec::new();
        path.write_buf(&mut buf);
        assert_eq_text!(buf, b"");
    }

    #[test]
    fn gen_path_6() {
        let path = Path::from_data(b"M 10 20 L 30 40 L 50 60 l 70 80").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.remove_duplicated_commands = true;

        let mut buf = Vec::new();
        path.write_buf_opt(&opt, &mut buf);
        assert_eq_text!(buf, b"M 10 20 L 30 40 50 60 l 70 80");
    }

    #[test]
    fn gen_path_7() {
        let path = Path::from_data(b"M 10 20 30 40 50 60").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.remove_duplicated_commands = true;

        let mut buf = Vec::new();
        path.write_buf_opt(&opt, &mut buf);
        assert_eq_text!(buf, b"M 10 20 L 30 40 50 60");
    }

    #[test]
    fn gen_path_8() {
        let path = Path::from_data(b"M 10 20 L 30 40").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;

        let mut buf = Vec::new();
        path.write_buf_opt(&opt, &mut buf);
        assert_eq_text!(buf, b"M10 20L30 40");
    }

    #[test]
    fn gen_path_9() {
        let path = Path::from_data(b"M 10 20 V 30 H 40 V 50 H 60 Z").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;

        let mut buf = Vec::new();
        path.write_buf_opt(&opt, &mut buf);
        assert_eq_text!(buf, b"M10 20V30H40V50H60Z");
    }

    #[test]
    fn gen_path_10() {
        let path = Path::from_data(b"M 10 -20 A 5.5 0.3 -4 1 1 0 -0.1").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;
        opt.numbers.remove_leading_zero = true;

        let mut buf = Vec::new();
        path.write_buf_opt(&opt, &mut buf);
        assert_eq_text!(buf, b"M10-20A5.5.3-4 1 1 0-.1");
        // TODO: this
        // assert_eq_text!(buf, b"M10-20A5.5.3-4 110-.1");
    }

    // TODO: M L L L -> M
}
