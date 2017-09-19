// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[cfg(feature = "parsing")]
use svgparser;
#[cfg(feature = "parsing")]
use svgparser::{
    Error as ParseError,
    TextFrame,
    Tokenize,
};

#[cfg(feature = "parsing")]
use FromStream;

use super::{
    Path,
    Segment,
    SegmentData,
};

#[cfg(feature = "parsing")]
impl FromStream for Path {
    type Err = ParseError;

    fn from_stream(s: TextFrame) -> Result<Path, ParseError> {
        use svgparser::path::Token;

        let mut tokens = svgparser::path::Tokenizer::from_frame(s).tokens();
        let mut p = Path::new();

        for seg in &mut tokens {
            let seg = match seg {
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
            };

            p.d.push(seg);
        }

        tokens.error()?;

        Ok(p)
    }
}
