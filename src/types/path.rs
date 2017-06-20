// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains all struct's for manipulating SVG [path data].
//!
//! [path data]: https://www.w3.org/TR/SVG/paths.html#PathData

// TODO: split into submodules

use std::{
    fmt,
    str,
};

#[cfg(feature = "parsing")]
use FromStream;
#[cfg(feature = "parsing")]
use svgparser;
#[cfg(feature = "parsing")]
use svgparser::{
    Error as ParseError,
    TextFrame,
    Tokenize,
};

use super::{number, FuzzyEq};
use {
    WriteBuffer,
    WriteOptions,
    WriteToString,
};

/// List of all path commands.
#[derive(Copy,Clone,Debug,PartialEq)]
#[allow(missing_docs)]
pub enum Command {
    MoveTo,
    LineTo,
    HorizontalLineTo,
    VerticalLineTo,
    CurveTo,
    SmoothCurveTo,
    Quadratic,
    SmoothQuadratic,
    EllipticalArc,
    ClosePath,
}

#[derive(Copy,Clone,Debug,PartialEq)]
#[allow(missing_docs)]
pub enum SegmentData {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    HorizontalLineTo {
        x: f64,
    },
    VerticalLineTo {
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    SmoothCurveTo {
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    Quadratic {
        x1: f64,
        y1: f64,
        x: f64,
        y: f64,
    },
    SmoothQuadratic {
        x: f64,
        y: f64,
    },
    EllipticalArc {
        rx: f64,
        ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64,
        y: f64,
    },
    ClosePath,
}

/// Representation of the path segment.
///
/// If you want to change the segment type (for example MoveTo to LineTo)
/// - you should create a new segment.
/// But you still can change points or make segment relative or absolute.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Segment {
    /// Indicate that segment is absolute.
    pub absolute: bool,
    data: SegmentData,
}

impl Segment {
    // TODO: to_relative
    // TODO: to_absolute

    /// Constructs a new MoveTo `Segment`.
    pub fn new_move_to(x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::MoveTo { x: x, y: y },
        }
    }

    /// Constructs a new ClosePath `Segment`.
    pub fn new_close_path() -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::ClosePath,
        }
    }

    /// Constructs a new LineTo `Segment`.
    pub fn new_line_to(x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::LineTo { x: x, y: y },
        }
    }

    /// Constructs a new HorizontalLineTo `Segment`.
    pub fn new_hline_to(x: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::HorizontalLineTo { x: x },
        }
    }

    /// Constructs a new VerticalLineTo `Segment`.
    pub fn new_vline_to(y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::VerticalLineTo { y: y },
        }
    }

    /// Constructs a new CurveTo `Segment`.
    pub fn new_curve_to(x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::CurveTo {
                x1: x1,
                y1: y1,
                x2: x2,
                y2: y2,
                x: x,
                y: y,
            },
        }
    }

    /// Constructs a new SmoothCurveTo `Segment`.
    pub fn new_smooth_curve_to(x2: f64, y2: f64, x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::SmoothCurveTo {
                x2: x2,
                y2: y2,
                x: x,
                y: y,
            },
        }
    }

    /// Constructs a new QuadTo `Segment`.
    pub fn new_quad_to(x1: f64, y1: f64, x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::Quadratic {
                x1: x1,
                y1: y1,
                x: x,
                y: y,
            },
        }
    }

    /// Constructs a new SmoothQuadTo `Segment`.
    pub fn new_smooth_quad_to(x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::SmoothQuadratic {
                x: x,
                y: y,
            },
        }
    }

    /// Constructs a new ArcTo `Segment`.
    pub fn new_arc_to(rx: f64, ry: f64, x_axis_rotation: f64, large_arc: bool, sweep: bool,
                  x: f64, y: f64) -> Segment {
        Segment {
            absolute: true,
            data: SegmentData::EllipticalArc {
                rx: rx,
                ry: ry,
                x_axis_rotation: x_axis_rotation,
                large_arc: large_arc,
                sweep: sweep,
                x: x,
                y: y,
            },
        }
    }

    /// Returns a segment type.
    pub fn cmd(&self) -> Command {
        match *self.data() {
            SegmentData::MoveTo { .. } => Command::MoveTo,
            SegmentData::LineTo { .. } => Command::LineTo,
            SegmentData::HorizontalLineTo { .. } => Command::HorizontalLineTo,
            SegmentData::VerticalLineTo { .. } => Command::VerticalLineTo,
            SegmentData::CurveTo { .. } => Command::CurveTo,
            SegmentData::SmoothCurveTo { .. } => Command::SmoothCurveTo,
            SegmentData::Quadratic { .. } => Command::Quadratic,
            SegmentData::SmoothQuadratic { .. } => Command::SmoothQuadratic,
            SegmentData::EllipticalArc { .. } => Command::EllipticalArc,
            SegmentData::ClosePath => Command::ClosePath,
        }
    }

    /// Returns segment's data.
    pub fn data(&self) -> &SegmentData {
        &self.data
    }

    /// Returns segment's mutable data.
    pub fn data_mut(&mut self) -> &mut SegmentData {
        &mut self.data
    }

    /// Returns `true` if the segment is absolute.
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    #[inline]
    /// Returns `true` if the segment is relative.
    pub fn is_relative(&self) -> bool {
        !self.absolute
    }

    /// Returns the `x` coordinate of the segment if it has one.
    pub fn x(&self) -> Option<f64> {
        match *self.data() {
              SegmentData::MoveTo { x, .. }
            | SegmentData::LineTo { x, .. }
            | SegmentData::HorizontalLineTo { x }
            | SegmentData::CurveTo { x, .. }
            | SegmentData::SmoothCurveTo { x, .. }
            | SegmentData::Quadratic { x, .. }
            | SegmentData::SmoothQuadratic { x, .. }
            | SegmentData::EllipticalArc { x, .. } => Some(x),

              SegmentData::VerticalLineTo { .. }
            | SegmentData::ClosePath => None,
        }
    }

    /// Returns the `y` coordinate of the segment if it has one.
    pub fn y(&self) -> Option<f64> {
        match *self.data() {
              SegmentData::MoveTo { y, .. }
            | SegmentData::LineTo { y, .. }
            | SegmentData::VerticalLineTo { y }
            | SegmentData::CurveTo { y, .. }
            | SegmentData::SmoothCurveTo { y, .. }
            | SegmentData::Quadratic { y, .. }
            | SegmentData::SmoothQuadratic { y, .. }
            | SegmentData::EllipticalArc { y, .. } => Some(y),

              SegmentData::HorizontalLineTo { .. }
            | SegmentData::ClosePath => None,
        }
    }

    /// Compares two segments using fuzzy float compare algorithm.
    ///
    /// Use it instead of `==`.
    ///
    /// It's not very fast.
    pub fn fuzzy_eq(&self, other: &Segment) -> bool {
        if self.absolute != other.absolute {
            return false;
        }

        use self::SegmentData as Seg;

        // TODO: find a way to wrap it in macro
        match (self.data, other.data) {
            (Seg::MoveTo { x, y }, Seg::MoveTo { x: ox, y: oy }) |
            (Seg::LineTo { x, y }, Seg::LineTo { x: ox, y: oy }) |
            (Seg::SmoothQuadratic { x, y }, Seg::SmoothQuadratic { x: ox, y: oy }) => {
                x.fuzzy_eq(&ox) && y.fuzzy_eq(&oy)
            }
            (Seg::HorizontalLineTo { x }, Seg::HorizontalLineTo { x: ox }) => {
                x.fuzzy_eq(&ox)
            }
            (Seg::VerticalLineTo { y }, Seg::VerticalLineTo { y: oy }) => {
                y.fuzzy_eq(&oy)
            }
            (Seg::CurveTo { x1, y1, x2, y2, x, y },
                Seg::CurveTo { x1: ox1, y1: oy1, x2: ox2, y2: oy2, x: ox, y: oy }) => {
                   x.fuzzy_eq(&ox)   && y.fuzzy_eq(&oy)
                && x1.fuzzy_eq(&ox1) && y1.fuzzy_eq(&oy1)
                && x2.fuzzy_eq(&ox2) && y2.fuzzy_eq(&oy2)
            }
            (Seg::SmoothCurveTo { x2, y2, x, y },
                Seg::SmoothCurveTo { x2: ox2, y2: oy2, x: ox, y: oy }) => {
                   x.fuzzy_eq(&ox)   && y.fuzzy_eq(&oy)
                && x2.fuzzy_eq(&ox2) && y2.fuzzy_eq(&oy2)
            }
            (Seg::Quadratic { x1, y1, x, y },
                Seg::Quadratic { x1: ox1, y1: oy1, x: ox, y: oy }) => {
                   x.fuzzy_eq(&ox)   && y.fuzzy_eq(&oy)
                && x1.fuzzy_eq(&ox1) && y1.fuzzy_eq(&oy1)
            }
            (Seg::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, x, y },
                Seg::EllipticalArc { rx: orx, ry: ory, x_axis_rotation: ox_axis_rotation,
                                     large_arc: olarge_arc, sweep: osweep, x: ox, y: oy }) => {
                   x.fuzzy_eq(&ox)   && y.fuzzy_eq(&oy)
                && rx.fuzzy_eq(&orx) && ry.fuzzy_eq(&ory)
                && x_axis_rotation == ox_axis_rotation
                && large_arc == olarge_arc
                && sweep == osweep
            }
            (Seg::ClosePath, Seg::ClosePath) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf: Vec<u8> = Vec::new();
        let mut flag = false;
        let opt = WriteOptions::default();

        write_cmd_char(self, &mut buf);
        buf.push(b' ');

        match *self.data() {
              SegmentData::MoveTo { x, y }
            | SegmentData::LineTo { x, y }
            | SegmentData::SmoothQuadratic { x, y } => {
                write_coords(&[x, y], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::HorizontalLineTo { x } => {
                write_coords(&[x], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::VerticalLineTo { y } => {
                write_coords(&[y], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::CurveTo { x1, y1, x2, y2, x, y } => {
                write_coords(&[x1, y1, x2, y2, x, y], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::SmoothCurveTo { x2, y2, x, y } => {
                write_coords(&[x2, y2, x, y], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::Quadratic { x1, y1, x, y } => {
                write_coords(&[x1, y1, x, y], true, &mut flag, &opt, &mut buf);
            }

            SegmentData::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                write_coords(&[rx, ry, x_axis_rotation], true, &mut flag, &opt, &mut buf);
                buf.push(b' ');
                write_flag(large_arc, &mut buf);
                buf.push(b' ');
                write_flag(sweep, &mut buf);
                buf.push(b' ');

                write_coords(&[x, y], true, &mut flag, &opt, &mut buf);
            }
            SegmentData::ClosePath => {},
        }

        // remove trailing space
        buf.pop();

        write!(f, "{}", str::from_utf8(&buf).unwrap())
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;

    macro_rules! test_seg {
        ($name:ident,  $seg1:expr, $seg2:expr) => (
        #[test]
        fn $name() {
            assert!($seg1 != $seg2);
            assert!($seg1.fuzzy_eq(&$seg2));
        })
    }

    test_seg!(test_fuzzy_eq_m,
        Segment::new_move_to(10.0, 10.1 + 10.2),
        Segment::new_move_to(10.0, 20.3)
    );

    test_seg!(test_fuzzy_eq_l,
        Segment::new_line_to(10.0, 10.1 + 10.2),
        Segment::new_line_to(10.0, 20.3)
    );

    test_seg!(test_fuzzy_eq_h,
        Segment::new_hline_to(10.1 + 10.2),
        Segment::new_hline_to(20.3)
    );

    test_seg!(test_fuzzy_eq_v,
        Segment::new_vline_to(10.1 + 10.2),
        Segment::new_vline_to(20.3)
    );

    test_seg!(test_fuzzy_eq_c,
        Segment::new_curve_to(10.0, 10.1 + 10.2, 10.0, 10.0, 10.0, 10.0),
        Segment::new_curve_to(10.0, 20.3, 10.0, 10.0, 10.0, 10.0)
    );

    test_seg!(test_fuzzy_eq_s,
        Segment::new_smooth_curve_to(10.0, 10.1 + 10.2, 10.0, 10.0),
        Segment::new_smooth_curve_to(10.0, 20.3, 10.0, 10.0)
    );

    test_seg!(test_fuzzy_eq_q,
        Segment::new_smooth_curve_to(10.0, 10.1 + 10.2, 10.0, 10.0),
        Segment::new_smooth_curve_to(10.0, 20.3, 10.0, 10.0)
    );

    test_seg!(test_fuzzy_eq_t,
        Segment::new_smooth_quad_to(10.0, 10.1 + 10.2),
        Segment::new_smooth_quad_to(10.0, 20.3)
    );

    test_seg!(test_fuzzy_eq_a,
        Segment::new_arc_to(10.0, 10.0, 0.0, true, true, 10.0, 10.1 + 10.2),
        Segment::new_arc_to(10.0, 10.0, 0.0, true, true, 10.0, 20.3)
    );

    #[test]
    fn test_fuzzy_ne_1() {
        let seg1 = Segment::new_move_to(10.0, 10.0);
        let seg2 = Segment::new_move_to(10.0, 20.0);

        assert!(seg1 != seg2);
        assert!(!seg1.fuzzy_eq(&seg2));
    }
}


/// Representation of the SVG path data.
#[derive(Default,PartialEq,Clone)]
pub struct Path {
    /// Vector which contain all segments.
    pub d: Vec<Segment>
}

impl Path {
    /// Constructs a new path.
    pub fn new() -> Path {
        Path { d: Vec::new() }
    }

    /// Constructs a new path with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Path {
        Path { d: Vec::with_capacity(capacity) }
    }

    // TODO: append Path

    /// Converts path's segments into absolute one.
    ///
    /// Original segments can be mixed (relative, absolute).
    pub fn conv_to_absolute(&mut self) {
        // position of the previous segment
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        // Position of the previous MoveTo segment.
        // When we get 'm'(relative) segment, which is not first segment - then it's
        // relative to previous 'M'(absolute) segment, not to first segment.
        let mut prev_mx = 0.0;
        let mut prev_my = 0.0;

        let mut prev_cmd = Command::MoveTo;
        for seg in &mut self.d {
            if seg.cmd() == Command::ClosePath {
                prev_x = prev_mx;
                prev_y = prev_my;

                seg.absolute = true;
                continue;
            }

            let offset_x;
            let offset_y;
            if seg.is_relative() {
                if seg.cmd() == Command::MoveTo && prev_cmd == Command::ClosePath {
                    offset_x = prev_mx;
                    offset_y = prev_my;
                } else {
                    offset_x = prev_x;
                    offset_y = prev_y;
                }
            } else {
                offset_x = 0.0;
                offset_y = 0.0;
            }

            if seg.is_relative() {
                shift_segment_data(seg.data_mut(), offset_x, offset_y);
            }

            if seg.cmd() == Command::MoveTo {
                prev_mx = seg.x().unwrap();
                prev_my = seg.y().unwrap();
            }

            seg.absolute = true;

            if seg.cmd() == Command::HorizontalLineTo {
                prev_x = seg.x().unwrap();
            } else if seg.cmd() == Command::VerticalLineTo {
                prev_y = seg.y().unwrap();
            } else {
                prev_x = seg.x().unwrap();
                prev_y = seg.y().unwrap();
            }

            prev_cmd = seg.cmd();
        }
    }

    /// Converts path's segments into relative one.
    ///
    /// Original segments can be mixed (relative, absolute).
    pub fn conv_to_relative(&mut self) {
        // NOTE: this method may look like 'conv_to_absolute', but it's a bit different.

        // position of the previous segment
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        // Position of the previous MoveTo segment.
        // When we get 'm'(relative) segment, which is not first segment - then it's
        // relative to previous 'M'(absolute) segment, not to first segment.
        let mut prev_mx = 0.0;
        let mut prev_my = 0.0;

        let mut prev_cmd = Command::MoveTo;
        for seg in &mut self.d {
            if seg.cmd() == Command::ClosePath {
                prev_x = prev_mx;
                prev_y = prev_my;

                seg.absolute = false;
                continue;
            }

            let offset_x;
            let offset_y;
            if seg.is_absolute() {
                if seg.cmd() == Command::MoveTo && prev_cmd == Command::ClosePath {
                    offset_x = prev_mx;
                    offset_y = prev_my;
                } else {
                    offset_x = prev_x;
                    offset_y = prev_y;
                }
            } else {
                offset_x = 0.0;
                offset_y = 0.0;
            }

            // unlike in 'to_absolute', we should take prev values before changing segment data
            if seg.is_absolute() {
                if seg.cmd() == Command::HorizontalLineTo {
                    prev_x = seg.x().unwrap();
                } else if seg.cmd() == Command::VerticalLineTo {
                    prev_y = seg.y().unwrap();
                } else {
                    prev_x = seg.x().unwrap();
                    prev_y = seg.y().unwrap();
                }
            } else {
                if seg.cmd() == Command::HorizontalLineTo {
                    prev_x += seg.x().unwrap();
                } else if seg.cmd() == Command::VerticalLineTo {
                    prev_y += seg.y().unwrap();
                } else {
                    prev_x += seg.x().unwrap();
                    prev_y += seg.y().unwrap();
                }
            }

            if seg.cmd() == Command::MoveTo {
                if seg.is_absolute() {
                    prev_mx = seg.x().unwrap();
                    prev_my = seg.y().unwrap();
                } else {
                    prev_mx += seg.x().unwrap();
                    prev_my += seg.y().unwrap();
                }
            }

            if seg.is_absolute() {
                shift_segment_data(seg.data_mut(), -offset_x, -offset_y);
            }

            seg.absolute = false;

            prev_cmd = seg.cmd();
        }
    }
}

fn shift_segment_data(d: &mut SegmentData, offset_x: f64, offset_y: f64) {
    match *d {
        SegmentData::MoveTo { ref mut x, ref mut y } => {
            *x += offset_x;
            *y += offset_y;
        }
        SegmentData::LineTo { ref mut x, ref mut y } => {
            *x += offset_x;
            *y += offset_y;
        }
        SegmentData::HorizontalLineTo { ref mut x } => {
            *x += offset_x;
        }
        SegmentData::VerticalLineTo { ref mut y } => {
            *y += offset_y;
        }
        SegmentData::CurveTo { ref mut x1, ref mut y1, ref mut x2, ref mut y2,
                               ref mut x, ref mut y } => {
            *x1 += offset_x;
            *y1 += offset_y;
            *x2 += offset_x;
            *y2 += offset_y;
            *x  += offset_x;
            *y  += offset_y;
        }
        SegmentData::SmoothCurveTo { ref mut x2, ref mut y2, ref mut x, ref mut y } => {
            *x2 += offset_x;
            *y2 += offset_y;
            *x  += offset_x;
            *y  += offset_y;
        }
        SegmentData::Quadratic { ref mut x1, ref mut y1, ref mut x, ref mut y } => {
            *x1 += offset_x;
            *y1 += offset_y;
            *x  += offset_x;
            *y  += offset_y;
        }
        SegmentData::SmoothQuadratic { ref mut x, ref mut y } => {
            *x += offset_x;
            *y += offset_y;
        }
        SegmentData::EllipticalArc { ref mut x, ref mut y, .. } => {
            *x += offset_x;
            *y += offset_y;
        }
        SegmentData::ClosePath => {}
    }
}

/// Construct a new path using build pattern.
#[derive(Default)]
pub struct Builder {
    path: Path,
}

impl Builder {
    /// Constructs a new builder.
    pub fn new() -> Builder {
        Builder { path: Path::new() }
    }

    /// Constructs a new builder with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Builder {
        Builder { path: Path::with_capacity(capacity) }
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

#[cfg(feature = "parsing")]
impl FromStream for Path {
    type Err = ParseError;

    fn from_stream(s: TextFrame) -> Result<Path, ParseError> {
        use svgparser::path::Token;

        let mut t = svgparser::path::Tokenizer::from_frame(s);
        let mut p = Path::new();

        loop {
            let seg = match t.parse_next()? {
                Token::MoveTo { abs, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::MoveTo { x: x, y: y },
                    }
                }
                Token::LineTo { abs, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::LineTo { x: x, y: y },
                    }
                }
                Token::HorizontalLineTo { abs, x } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::HorizontalLineTo { x: x },
                    }
                }
                Token::VerticalLineTo { abs, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::VerticalLineTo { y: y },
                    }
                }
                Token::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::CurveTo { x1: x1, y1: y1, x2: x2, y2: y2, x: x, y: y },
                    }
                }
                Token::SmoothCurveTo { abs, x2, y2, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::SmoothCurveTo { x2: x2, y2: y2, x: x, y: y },
                    }
                }
                Token::Quadratic { abs, x1, y1, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::Quadratic { x1: x1, y1: y1, x: x, y: y },
                    }
                }
                Token::SmoothQuadratic { abs, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::SmoothQuadratic { x: x, y: y },
                    }
                }
                Token::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::EllipticalArc {
                            rx: rx, ry: ry,
                            x_axis_rotation: x_axis_rotation,
                            large_arc: large_arc, sweep: sweep,
                            x: x, y: y
                        },
                    }
                }
                Token::ClosePath { abs } => {
                    Segment {
                        absolute: abs,
                        data: SegmentData::ClosePath,
                    }
                }
                Token::EndOfStream => {
                    break;
                }
            };

            p.d.push(seg);
        }

        Ok(p)
    }
}

struct PrevCmd {
    cmd: Command,
    absolute: bool,
    implicit: bool,
}

impl WriteBuffer for Path {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        if self.d.is_empty() {
            return;
        }

        let mut prev_cmd: Option<PrevCmd> = None;
        let mut prev_coord_has_dot = false;

        for seg in &self.d {
            let is_written = write_cmd(seg, &mut prev_cmd, opt, buf);
            match *seg.data() {
                  SegmentData::MoveTo { x, y }
                | SegmentData::LineTo { x, y }
                | SegmentData::SmoothQuadratic { x, y } => {
                    write_coords(&[x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::HorizontalLineTo { x } => {
                    write_coords(&[x], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::VerticalLineTo { y } => {
                    write_coords(&[y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::CurveTo { x1, y1, x2, y2, x, y } => {
                    write_coords(&[x1, y1, x2, y2, x, y], is_written,
                                 &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::SmoothCurveTo { x2, y2, x, y } => {
                    write_coords(&[x2, y2, x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::Quadratic { x1, y1, x, y } => {
                    write_coords(&[x1, y1, x, y], is_written, &mut prev_coord_has_dot, opt, buf);
                }

                SegmentData::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                    write_coords(&[rx, ry, x_axis_rotation], is_written,
                                 &mut prev_coord_has_dot, opt, buf);

                    if opt.paths.use_compact_notation {
                        // flags must always have a space before it
                        buf.push(b' ');
                    }

                    write_flag(large_arc, buf);
                    if !opt.paths.join_arc_to_flags {
                        buf.push(b' ');
                    }
                    write_flag(sweep, buf);
                    if !opt.paths.join_arc_to_flags {
                        buf.push(b' ');
                    }

                    // reset, because flags can't have dots
                    prev_coord_has_dot = false;

                    // 'is_explicit_cmd' is always 'true'
                    // because it's relevant only for first coordinate of the segment
                    write_coords(&[x, y], true, &mut prev_coord_has_dot, opt, buf);
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

fn write_cmd(seg: &Segment, prev_cmd: &mut Option<PrevCmd>,
             opt: &WriteOptions, buf: &mut Vec<u8>) -> bool {

    let mut print_cmd = true;
    if opt.paths.remove_duplicated_commands {
        // check that previous command is the same as current
        if let Some(ref pcmd) = *prev_cmd {
            if seg.cmd() == pcmd.cmd && seg.absolute == pcmd.absolute {
                print_cmd = false;
            }
        }
    }

    let mut is_implicit = false;
    if opt.paths.use_implicit_lineto_commands {

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

    if !(seg.cmd() == Command::ClosePath || opt.paths.use_compact_notation) {
        buf.push(b' ');
    }

    true
}

fn write_cmd_char(seg: &Segment, buf: &mut Vec<u8>) {
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

fn write_coords(coords: &[f64], is_explicit_cmd: bool, prev_coord_has_dot: &mut bool,
                opt: &WriteOptions, buf: &mut Vec<u8>)
{
    if opt.paths.use_compact_notation {
        for (i, num) in coords.iter().enumerate() {
            let start_pos = buf.len() - 1;

            number::write_num(num, opt.remove_leading_zero, buf);

            let c = buf[start_pos + 1];

            let write_space;
            if !*prev_coord_has_dot && c == b'.' {
                write_space = true;
            } else if i == 0 && is_explicit_cmd {
                write_space = false;
            } else if (c as char).is_digit(10) {
                write_space = true;
            } else {
                write_space = false;
            }

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

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::path;
    use {WriteOptions, FromStream, WriteToString};

    #[test]
    fn gen_path_1() {
        let mut path = Path::new();
        path.d.push(path::Segment::new_move_to(10.0, 20.0));
        path.d.push(path::Segment::new_line_to(10.0, 20.0));
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
                opt.paths.$flag = true;

                assert_eq_text!(path.to_string_with_opt(&opt), $out_text);
            }
        )
    }

    test_gen_path_opt!(gen_path_6,
        "M 10 20 L 30 40 L 50 60 l 70 80",
        "M 10 20 L 30 40 50 60 l 70 80",
        remove_duplicated_commands);

    test_gen_path_opt!(gen_path_7,
        "M 10 20 30 40 50 60",
        "M 10 20 L 30 40 50 60",
        remove_duplicated_commands);

    test_gen_path_opt!(gen_path_8,
        "M 10 20 L 30 40",
        "M10 20L30 40",
        use_compact_notation);

    test_gen_path_opt!(gen_path_9,
        "M 10 20 V 30 H 40 V 50 H 60 Z",
        "M10 20V30H40V50H60Z",
        use_compact_notation);

    #[test]
    fn gen_path_10() {
        let path = Path::from_str("M 10 -20 A 5.5 0.3 -4 1 1 0 -0.1").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;
        opt.paths.join_arc_to_flags = true;
        opt.remove_leading_zero = true;

        assert_eq_text!(path.to_string_with_opt(&opt), "M10-20A5.5.3-4 110-.1");
    }

    test_gen_path_opt!(gen_path_11,
        "M 10-10 a 1 1 0 1 1 -1 1",
        "M10-10a1 1 0 1 1 -1 1",
        use_compact_notation);

    test_gen_path_opt!(gen_path_12,
        "M 10-10 a 1 1 0 1 1 0.1 1",
        "M10-10a1 1 0 1 1 0.1 1",
        use_compact_notation);

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
        opt.paths.use_implicit_lineto_commands = true;
        opt.paths.remove_duplicated_commands = true;

        assert_eq_text!(path.to_string_with_opt(&opt), "M 10 20 30 40 50 60 M 10 20 30 40 50 60");
    }

    #[test]
    fn gen_path_19() {
        let path = Path::from_str("m10 20 A 10 10 0 1 0 0 0 A 2 2 0 1 0 2 0").unwrap();

        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;
        opt.paths.remove_duplicated_commands = true;
        opt.remove_leading_zero = true;

        // may generate as 'm10 20A10 10 0 1 0 0 0 2 2 0 1 0  2 0' <- two spaces

        assert_eq_text!(path.to_string_with_opt(&opt), "m10 20A10 10 0 1 0 0 0 2 2 0 1 0 2 0");
    }

    #[test]
    fn test_seg_display_1() {
        let mut seg = path::Segment::new_move_to(10.0, 20.0);
        assert_eq_text!(format!("{}", seg), "M 10 20");

        seg.absolute = false;
        assert_eq_text!(format!("{}", seg), "m 10 20");

        assert_eq_text!(format!("{}", path::Segment::new_close_path()), "Z");
    }
}

#[cfg(test)]
mod to_absolute {
    use types::path;
    use FromStream;

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let mut path = path::Path::from_str($in_text).unwrap();
                path.conv_to_absolute();
                assert_eq_text!(path.to_string(), $out_text);
            }
        )
    }

    test!(line_to,
          "m 10 20 l 20 20",
          "M 10 20 L 30 40");

    test!(close_path,
          "m 10 20 l 20 20 z",
          "M 10 20 L 30 40 Z");

    // test to check that libsvgparser parses implicit MoveTo as LineTo
    test!(implicit_line_to,
          "m 10 20 20 20",
          "M 10 20 L 30 40");

    test!(hline_vline,
          "m 10 20 v 10 h 10 l 10 10",
          "M 10 20 V 30 H 20 L 30 40");

    test!(curve,
          "m 10 20 c 10 10 10 10 10 10",
          "M 10 20 C 20 30 20 30 20 30");

    test!(move_to_1,
          "m 10 20 l 10 10 m 10 10 l 10 10",
          "M 10 20 L 20 30 M 30 40 L 40 50");

    test!(move_to_2,
          "m 10 20 l 10 10 z m 10 10 l 10 10",
          "M 10 20 L 20 30 Z M 20 30 L 30 40");

    test!(move_to_3,
          "m 10 20 l 10 10 Z m 10 10 l 10 10",
          "M 10 20 L 20 30 Z M 20 30 L 30 40");

    test!(smooth_curve,
          "m 10 20 s 10 10 10 10",
          "M 10 20 S 20 30 20 30");

    test!(quad,
          "m 10 20 q 10 10 10 10",
          "M 10 20 Q 20 30 20 30");

    test!(arc_mixed,
          "M 30 150 a 40 40 0 0 1 65 50 Z m 30 30 A 20 20 0 0 0 125 230 Z \
           m 40 24 a 20 20 0 0 1 65 50 z",
          "M 30 150 A 40 40 0 0 1 95 200 Z M 60 180 A 20 20 0 0 0 125 230 Z \
           M 100 204 A 20 20 0 0 1 165 254 Z");
}

#[cfg(test)]
mod to_relative {
    use types::path;
    use FromStream;

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let mut path = path::Path::from_str($in_text).unwrap();
                path.conv_to_relative();
                assert_eq_text!(path.to_string(), $out_text);
            }
        )
    }

    test!(line_to,
          "M 10 20 L 30 40",
          "m 10 20 l 20 20");

    test!(close_path,
          "M 10 20 L 30 40 Z",
          "m 10 20 l 20 20 z");

    test!(implicit_line_to,
          "M 10 20 30 40",
          "m 10 20 l 20 20");

    test!(hline_vline,
          "M 10 20 V 30 H 20 L 30 40",
          "m 10 20 v 10 h 10 l 10 10");

    test!(curve,
          "M 10 20 C 20 30 20 30 20 30",
          "m 10 20 c 10 10 10 10 10 10");

    test!(move_to_1,
          "M 10 20 L 20 30 M 30 40 L 40 50",
          "m 10 20 l 10 10 m 10 10 l 10 10");

    test!(move_to_2,
          "M 10 20 L 20 30 Z M 20 30 L 30 40",
          "m 10 20 l 10 10 z m 10 10 l 10 10");

    test!(move_to_3,
          "M 10 20 L 20 30 z M 20 30 L 30 40",
          "m 10 20 l 10 10 z m 10 10 l 10 10");

    test!(smooth_curve,
          "M 10 20 S 20 30 20 30",
          "m 10 20 s 10 10 10 10");

    test!(quad,
          "M 10 20 Q 20 30 20 30",
          "m 10 20 q 10 10 10 10");

    test!(arc_mixed,
          "M 30 150 a 40 40 0 0 1 65 50 Z m 30 30 A 20 20 0 0 0 125 230 Z \
           m 40 24 a 20 20 0 0 1 65 50 z",
          "m 30 150 a 40 40 0 0 1 65 50 z m 30 30 a 20 20 0 0 0 65 50 z \
           m 40 24 a 20 20 0 0 1 65 50 z");
}
