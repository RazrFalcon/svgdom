// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains all struct's for manipulating SVG [path data].
//!
//! [path data]: https://www.w3.org/TR/SVG/paths.html#PathData

pub use self::builder::*;
pub use self::path::*;
pub use self::segment::*;

mod builder;
mod parser;
mod path;
mod segment;
mod writer;
