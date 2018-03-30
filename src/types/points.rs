// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::str::FromStr;
use std::ops::{Deref, DerefMut};

use svgparser::{
    self,
    StreamError,
};

use svgparser::xmlparser::{
    FromSpan,
    StrSpan,
};

use {
    ListSeparator,
    ParseFromSpan,
    ToStringWithOptions,
    WriteBuffer,
    WriteOptions,
};

/// Representation of the SVG `points` attribute data.
#[derive(Clone, PartialEq, Debug)]
pub struct Points(Vec<(f64, f64)>);

impl Points {
    /// Constructs a new points container.
    pub fn new() -> Self {
        Points(Vec::new())
    }

    /// Constructs a new points container with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Points(Vec::with_capacity(capacity))
    }
}

impl_from_str!(Points);

impl ParseFromSpan for Points {
    fn from_span(span: StrSpan) -> Result<Points, Self::Err> {
        let tokenizer = svgparser::Points::from_span(span);
        let p: Vec<_> = tokenizer.collect();
        Ok(Points(p))
    }
}

impl WriteBuffer for (f64, f64) {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        self.0.write_buf_opt(opt, buf);

        match opt.list_separator {
            ListSeparator::Space => buf.push(b' '),
            ListSeparator::Comma => buf.push(b','),
            ListSeparator::CommaSpace => buf.extend_from_slice(b", "),
        }

        self.1.write_buf_opt(opt, buf);
    }
}

impl WriteBuffer for Points {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        if self.len() < 2 {
            return;
        }

        super::write_list(self, opt, buf);
    }
}

impl_display!(Points);

impl Deref for Points {
    type Target = Vec<(f64, f64)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Points {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use {
        ListSeparator,
        Points,
        ToStringWithOptions,
        WriteOptions,
    };

    #[test]
    fn parse_points_1() {
        let points = Points::from_str("10 20 30 40").unwrap();
        assert_eq!(*points, vec![(10.0, 20.0), (30.0, 40.0)]);
    }

    #[test]
    fn parse_points_2() {
        let points = Points::from_str("10 20 30 40 50").unwrap();
        assert_eq!(*points, vec![(10.0, 20.0), (30.0, 40.0)]);
    }

    #[test]
    fn parse_points_3() {
        let points = Points::from_str("10 20 30 40").unwrap();
        assert_eq!(points.to_string(), "10 20 30 40");
    }

    #[test]
    fn parse_points_4() {
        let points = Points::from_str("10 20 30 40").unwrap();

        let opt = WriteOptions {
            list_separator: ListSeparator::Comma,
            .. WriteOptions::default()
        };

        assert_eq!(points.to_string_with_opt(&opt), "10,20,30,40");
    }
}
