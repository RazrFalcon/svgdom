// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
