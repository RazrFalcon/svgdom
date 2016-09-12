// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::ops::Mul;
use std::f64;

use {WriteOptions, FromStream, WriteBuffer, WriteToString};
use super::number::write_num;

use svgparser::Error as ParseError;
use svgparser::Stream;

/// Representation of `<transform>` type.
#[derive(Debug,Clone,Copy)]
#[allow(missing_docs)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Transform {
    /// Constructs a new transform.
    pub fn new(a: f64, b: f64, c: f64, d: f64,  e: f64, f: f64) -> Transform {
        Transform {
            a: a,
            b: b,
            c: c,
            d: d,
            e: e,
            f: f,
        }
    }

    /// Translates the current transform.
    pub fn translate(mut self, x: f64, y: f64) -> Transform {
        self.append(&Transform::new(1.0, 0.0, 0.0, 1.0, x, y));
        self
    }

    /// Scales the current transform.
    pub fn scale(mut self, sx: f64, sy: f64) -> Transform {
        self.append(&Transform::new(sx, 0.0, 0.0, sy, 0.0, 0.0));
        self
    }

    // TODO: rotate
    // TODO: rotate by pos
    // TODO: skewX
    // TODO: skewY

    /// Appends transform to the current transform.
    pub fn append(&mut self, t: &Transform) {
        let tm = self.to_matrix() * t.to_matrix();
        self.a = tm.d[0][0];
        self.c = tm.d[1][0];
        self.e = tm.d[2][0];
        self.b = tm.d[0][1];
        self.d = tm.d[1][1];
        self.f = tm.d[2][1];
    }

    fn to_matrix(&self) -> TransformMatrix {
        let mut tm = TransformMatrix::default();
        tm.d[0][0] = self.a;
        tm.d[1][0] = self.c;
        tm.d[2][0] = self.e;
        tm.d[0][1] = self.b;
        tm.d[1][1] = self.d;
        tm.d[2][1] = self.f;
        tm
    }

    /// Returns `true` if the current transform is default, aka (1 0 0 1 0 0).
    pub fn is_default(&self) -> bool {
           self.a == 1.0
        && self.b == 0.0
        && self.c == 0.0
        && self.d == 1.0
        && self.e == 0.0
        && self.f == 0.0
    }

    /// Returns `true` if the current transform contains only translate part.
    pub fn is_translate(&self) -> bool {
           self.a == 1.0
        && self.b == 0.0
        && self.c == 0.0
        && self.d == 1.0
        && (self.e != 0.0 || self.f != 0.0)
    }

    /// Returns `true` if the current transform contains only scale part.
    pub fn is_scale(&self) -> bool {
           (self.a != 1.0 || self.d != 1.0)
        && self.b == 0.0
        && self.c == 0.0
        && self.e == 0.0
        && self.f == 0.0
    }

    /// Returns `true` if the current transform contains only proportional scale part.
    ///
    /// The proportional scale is when `<sx>` equal to `<sy>`.
    pub fn is_proportional_scale(&self) -> bool {
        self.is_scale() && self.a == self.d
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
           self.a == other.a
        && self.b == other.b
        && self.c == other.c
        && self.d == other.d
        && self.e == other.e
        && self.f == other.f
    }
}

impl FromStream for Transform {
    type Err = ParseError;

    fn from_stream(s: Stream) -> Result<Transform, ParseError> {
        use svgparser::transform::Tokenizer as TransformTokenizer;
        use svgparser::transform::Transform as ParserTransform;

        let mut ts = TransformTokenizer::new(s);

        let mut matrix = Transform::default();

        let pi = f64::consts::PI;

        while let Some(n) = ts.next() {
            match n {
                Ok(v) => {
                    match v {
                        ParserTransform::Matrix { a, b, c, d, e, f }
                            => matrix.append(&Transform::new(a, b, c, d, e, f)),
                        ParserTransform::Translate { tx, ty }
                            => matrix.append(&Transform::new(1.0, 0.0, 0.0, 1.0, tx, ty)),
                        ParserTransform::Scale { sx, sy }
                            => matrix.append(&Transform::new(sx, 0.0, 0.0, sy, 0.0, 0.0)),
                        ParserTransform::Rotate { angle } => {
                            let v = (angle / 180.0) * pi;
                            let a =  v.cos();
                            let b =  v.sin();
                            let c = -b;
                            let d =  a;
                            matrix.append(&Transform::new(a, b, c, d, 0.0, 0.0))
                        }
                        ParserTransform::SkewX { angle } => {
                            let c = ((angle / 180.0 ) * pi).tan();
                            matrix.append(&Transform::new(1.0, 0.0, c, 1.0, 0.0, 0.0))
                        },
                        ParserTransform::SkewY { angle } => {
                            let b = ((angle / 180.0) * pi).tan();
                            matrix.append(&Transform::new(1.0, b, 0.0, 1.0, 0.0, 0.0))
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(matrix)
    }
}

impl WriteBuffer for Transform {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        if self.is_default() {
            return;
        }

        if opt.transforms.simplify_matrix {
            write_simplified_transform(self, opt, buf);
        } else {
            write_matrix_transform(self, opt, buf);
        }
    }
}

fn write_matrix_transform(ts: &Transform, opt: &WriteOptions, out: &mut Vec<u8>) {
    let pt = opt.numbers.precision_transforms;
    let pc = opt.numbers.precision_coordinates;
    let rm = opt.numbers.remove_leading_zero;

    out.extend_from_slice(b"matrix(");
    write_num(ts.a, pt, rm, out);
    out.push(b' ');
    write_num(ts.b, pt, rm, out);
    out.push(b' ');
    write_num(ts.c, pt, rm, out);
    out.push(b' ');
    write_num(ts.d, pt, rm, out);
    out.push(b' ');
    write_num(ts.e, pc, rm, out);
    out.push(b' ');
    write_num(ts.f, pc, rm, out);
    out.push(b')');
}

fn write_simplified_transform(ts: &Transform, opt: &WriteOptions, out: &mut Vec<u8>) {
    let pt = opt.numbers.precision_transforms;
    let pc = opt.numbers.precision_coordinates;
    let rm = opt.numbers.remove_leading_zero;

    if ts.is_translate() {
        out.extend_from_slice(b"translate(");
        write_num(ts.e, pc, rm, out);

        if ts.f != 0.0 {
            out.push(b' ');
            write_num(ts.f, pc, rm, out);
        }

        out.push(b')');
    } else if ts.is_scale() {
        out.extend_from_slice(b"scale(");
        write_num(ts.a, pt, rm, out);

        if ts.a != ts.d {
            out.push(b' ');
            write_num(ts.d, pt, rm, out);
        }

        out.push(b')');
    } else {
        write_matrix_transform(ts, opt, out);
    }
}

impl_display!(Transform);

struct TransformMatrix {
    d: [[f64; 3]; 3] ,
}

impl Default for TransformMatrix {
    fn default() -> TransformMatrix {
        TransformMatrix {
            d: [[1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0]]
        }
    }
}

impl Mul for TransformMatrix {
    type Output = TransformMatrix;

    fn mul(self, other: TransformMatrix) -> TransformMatrix {
        let mut res = TransformMatrix::default();
        for row in 0..3 {
            for col in 0..3 {
                let mut sum = 0.0;
                for j in 0..3 {
                    sum = sum + self.d[j][row] * other.d[col][j];
                }
                res.d[col][row] = sum;
            }
        }
        return res;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use {FromStream, WriteOptions, WriteBuffer};

    #[test]
    fn from_data_1() {
        assert_eq!(Transform::from_data(b"translate(10 20)").unwrap(),
            Transform::default().translate(10.0, 20.0));
    }

    #[test]
    fn from_data_2() {
        assert_eq!(Transform::from_data(b"translate(10 20) scale(2, 3)").unwrap(),
            Transform::default().translate(10.0, 20.0).scale(2.0, 3.0));
    }

    macro_rules! test_ts {
        ($name:ident, $ts:expr, $simplify:expr, $result:expr) => (
            #[test]
            fn $name() {
                let mut opt = WriteOptions::default();
                opt.transforms.simplify_matrix = $simplify;
                let mut out = Vec::new();
                $ts.write_buf_opt(&opt, &mut out);
                assert_eq!(String::from_utf8(out).unwrap(), $result);
            }
        )
    }

    test_ts!(write_buf_1, Transform::default(), false, "");
    test_ts!(write_buf_2,
        Transform::new(2.0, 0.0, 0.0, 3.0, 20.0, 30.0), false,
        "matrix(2 0 0 3 20 30)"
    );
    test_ts!(write_buf_3,
        Transform::new(1.0, 0.0, 0.0, 1.0, 20.0, 30.0), true,
        "translate(20 30)"
    );
    test_ts!(write_buf_4,
        Transform::new(1.0, 0.0, 0.0, 1.0, 20.0, 0.0), true,
        "translate(20)"
    );
    test_ts!(write_buf_5,
        Transform::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0), true,
        "scale(2 3)"
    );
    test_ts!(write_buf_6,
        Transform::new(2.0, 0.0, 0.0, 2.0, 0.0, 0.0), true,
        "scale(2)"
    );

    // TODO: this
    // COMPARE(Transform::fromRotate(45).toString(opt), "rotate(45)");
    // COMPARE(Transform::fromRotate(32).toString(opt), "rotate(32)");
    // COMPARE(Transform::fromSkewX(33).toString(opt), "matrix(1 0 0.64940759 1 0 0)");
    // COMPARE(Transform::fromSkewY(33).toString(opt), "matrix(1 0.64940759 0 1 0 0)");

    // { "5",
    //         "matrix(-1 0 0 -1 0 0)",
    //         "scale(-1)",
    //     },

    //     { "6",
    //         "matrix(-1 0 0 1 0 0)",
    //         "scale(-1 1)",
    //     },

    //     { "7",
    //         "matrix(1 0 0 -1 0 0)",
    //         "scale(1 -1)",
    //     },

    //     { "8",
    //         "matrix(0 -1 1 0 0 0)",
    //         "rotate(-90)",
    //     },

    //     { "9",
    //         "matrix(0 1 -1 0 0 0)",
    //         "rotate(90)",
    //     },
}
