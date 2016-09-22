// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::WriteOptions;
use svgparser::Stream;

/// Trait for parsing data from data stream.
pub trait FromStream: Sized {
    /// Error type.
    type Err;

    /// Parses data from `Stream`.
    fn from_stream(s: Stream) -> Result<Self, Self::Err>;

    /// Parses data from `&[u8]`.
    fn from_data(data: &[u8]) -> Result<Self, Self::Err> {
        FromStream::from_stream(Stream::new(data))
    }
}

/// Trait for writing data to buffer.
pub trait WriteBuffer {
    /// Writes data to `Vec<u8>` buffer using specified WriteOptions.
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>);

    /// Writes data to `Vec<u8>` buffer using default WriteOptions.
    fn write_buf(&self, buf: &mut Vec<u8>) {
        self.write_buf_opt(&WriteOptions::default(), buf);
    }
}

/// Trait for writing data to `String`. Tunable `to_string()` alternative.
pub trait WriteToString: WriteBuffer {
    /// Writes data to `String` using specified WriteOptions.
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
                let mut out = Vec::with_capacity(32);
                self.write_buf(&mut out);
                write!(f, "{}", String::from_utf8(out).unwrap())
            }
        }

        impl WriteToString for $t {}
    )
}
