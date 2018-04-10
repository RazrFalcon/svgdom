// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::FromStr;

use svgparser::xmlparser::{
    StrSpan,
};

use WriteOptions;

/// A trait for parsing data from a string.
pub trait ParseFromSpan: FromStr {
    /// Parses data from a `StrSpan`.
    fn from_span(s: StrSpan) -> Result<Self, <Self as FromStr>::Err>;
}

macro_rules! impl_from_str {
    ($t:ty) => (
        impl FromStr for $t {
            type Err = StreamError;

            fn from_str(s: &str) -> Result<$t, Self::Err> {
                ParseFromSpan::from_span(StrSpan::from_str(s))
            }
        }
    )
}

/// A trait for writing a data to the buffer.
pub trait WriteBuffer {
    /// Writes data to the `Vec<u8>` buffer using specified `WriteOptions`.
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>);

    /// Writes data to the `Vec<u8>` buffer using default `WriteOptions`.
    fn write_buf(&self, buf: &mut Vec<u8>) {
        self.write_buf_opt(&WriteOptions::default(), buf);
    }
}

/// A trait for converting a value to a `String` with `WriteOptions`.
///
/// A tunable `to_string()` alternative.
pub trait ToStringWithOptions: WriteBuffer {
    /// Writes data to the `String` using specified `WriteOptions`.
    fn to_string_with_opt(&self, opt: &WriteOptions) -> String {
        let mut out = Vec::with_capacity(32);
        self.write_buf_opt(opt, &mut out);
        String::from_utf8(out).unwrap()
    }
}

macro_rules! impl_display {
    ($t:ty) => (
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use std::str;

                let mut out = Vec::with_capacity(32);
                self.write_buf(&mut out);
                write!(f, "{}", str::from_utf8(&out).unwrap())
            }
        }

        impl ToStringWithOptions for $t {}
    )
}
