// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains submodules which represent SVG value types.

pub use self::transform::Transform;
pub use self::color::Color;
pub use self::length::Length;
pub use self::number::{FuzzyEq, FuzzyOrd};

pub use svgparser::LengthUnit;

/// Representation of the `<list-of-numbers>` type.
pub type NumberList = Vec<f64>;
/// Representation of the `<list-of-lengths>` type.
pub type LengthList = Vec<Length>;

pub mod path;
mod color;
mod length;
mod number;
mod transform;
