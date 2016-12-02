// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains submodules which represent SVG value types.

pub use self::transform::Transform;
pub use self::color::Color;
pub use self::length::Length;
pub use self::number::{FuzzyEq, FuzzyOrd};

pub use svgparser::LengthUnit;

pub use attribute::NumberList;
pub use attribute::LengthList;

pub mod path;
mod color;
mod length;
mod number;
mod transform;
