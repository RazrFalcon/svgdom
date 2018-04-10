// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains submodules which represent SVG value types.

pub use self::color::Color;
pub use self::length::Length;
pub use self::number::{FuzzyEq, FuzzyOrd};
pub use self::points::Points;
pub use self::transform::Transform;

pub use svgparser::{
    Align,
    AspectRatio,
    LengthUnit,
    ViewBox,
};

use {
    ListSeparator,
    ToStringWithOptions,
    WriteBuffer,
    WriteOptions,
};

/// Representation of the `<list-of-numbers>` type.
pub type NumberList = Vec<f64>;
/// Representation of the `<list-of-lengths>` type.
pub type LengthList = Vec<Length>;

pub mod path;
mod color;
mod length;
mod number;
mod points;
mod transform;


impl WriteBuffer for NumberList {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        write_list(self, opt, buf);
    }
}

impl WriteBuffer for LengthList {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        write_list(self, opt, buf);
    }
}

// We can't use `impl_display` macro, because the `Display` trait
// can't be implement for a std type.
impl ToStringWithOptions for NumberList {}
impl ToStringWithOptions for LengthList {}

fn write_list<T: WriteBuffer>(list: &[T], opt: &WriteOptions, buf: &mut Vec<u8>) {
    for (n, l) in list.iter().enumerate() {
        l.write_buf_opt(opt, buf);
        if n < list.len() - 1 {
            match opt.list_separator {
                ListSeparator::Space => buf.push(b' '),
                ListSeparator::Comma => buf.push(b','),
                ListSeparator::CommaSpace => buf.extend_from_slice(b", "),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {WriteOptions, ToStringWithOptions, ListSeparator};

    #[test]
    fn write_list_1() {
        let list = vec![1.0, 2.0, 3.0];

        let mut opt = WriteOptions::default();
        opt.list_separator = ListSeparator::Space;

        assert_eq!(list.to_string_with_opt(&opt), "1 2 3");
    }

    #[test]
    fn write_list_2() {
        let list = vec![1.0, 2.0, 3.0];

        let mut opt = WriteOptions::default();
        opt.list_separator = ListSeparator::Comma;

        assert_eq!(list.to_string_with_opt(&opt), "1,2,3");
    }

    #[test]
    fn write_list_3() {
        let list = vec![1.0, 2.0, 3.0];

        let mut opt = WriteOptions::default();
        opt.list_separator = ListSeparator::CommaSpace;

        assert_eq!(list.to_string_with_opt(&opt), "1, 2, 3");
    }
}
