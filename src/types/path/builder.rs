// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{
    Path,
    Segment,
};

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
        self.path.push(Segment::new_move_to(x, y));
        self
    }

    /// Appends a new ClosePath segment.
    pub fn close_path(mut self) -> Builder {
        self.path.push(Segment::new_close_path());
        self
    }

    /// Appends a new LineTo segment.
    pub fn line_to(mut self, x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_line_to(x, y));
        self
    }

    /// Appends a new HorizontalLineTo segment.
    pub fn hline_to(mut self, x: f64) -> Builder {
        self.path.push(Segment::new_hline_to(x));
        self
    }

    /// Appends a new VerticalLineTo segment.
    pub fn vline_to(mut self, y: f64) -> Builder {
        self.path.push(Segment::new_vline_to(y));
        self
    }

    /// Appends a new CurveTo segment.
    pub fn curve_to(mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_curve_to(x1, y1, x2, y2, x, y));
        self
    }

    /// Appends a new SmoothCurveTo segment.
    pub fn smooth_curve_to(mut self, x2: f64, y2: f64, x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_smooth_curve_to(x2, y2, x, y));
        self
    }

    /// Appends a new QuadTo segment.
    pub fn quad_to(mut self, x1: f64, y1: f64, x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_quad_to(x1, y1, x, y));
        self
    }

    /// Appends a new SmoothQuadTo segment.
    pub fn smooth_quad_to(mut self, x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_smooth_quad_to(x, y));
        self
    }

    /// Appends a new ArcTo segment.
    pub fn arc_to(mut self, rx: f64, ry: f64, x_axis_rotation: f64, large_arc: bool, sweep: bool,
                  x: f64, y: f64) -> Builder {
        self.path.push(Segment::new_arc_to(rx, ry, x_axis_rotation, large_arc, sweep, x, y));
        self
    }

    /// Finalizes the build.
    pub fn finalize(self) -> Path {
        self.path
    }
}
