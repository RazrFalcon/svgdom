// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    fmt,
    str,
};

use super::writer;

use types::FuzzyEq;

use WriteOptions;

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
    /// Segment data.
    pub data: SegmentData,
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
        let mut opt = WriteOptions::default();
        opt.paths.use_compact_notation = true;

        writer::write_cmd_char(self, &mut buf);

        if self.cmd() != Command::ClosePath {
            buf.push(b' ');
        }

        writer::write_segment(self.data(), true, &mut false, &opt, &mut buf);

        write!(f, "{}", str::from_utf8(&buf).unwrap())
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_seg_display_1() {
        let mut seg = Segment::new_move_to(10.0, 20.0);
        assert_eq_text!(format!("{}", seg), "M 10 20");

        seg.absolute = false;
        assert_eq_text!(format!("{}", seg), "m 10 20");

        assert_eq_text!(format!("{}", Segment::new_close_path()), "Z");
    }
}
