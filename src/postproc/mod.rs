// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains functions that can be useful for SVG document post-processing.

pub use self::gradients::resolve_linear_gradient_attributes;
pub use self::gradients::resolve_radial_gradient_attributes;
pub use self::gradients::resolve_stop_attributes;

pub use self::fix_attrs::fix_rect_attributes;
pub use self::fix_attrs::fix_poly_attributes;
pub use self::fix_attrs::fix_stop_attributes;

pub use self::resolve_inherit::resolve_inherit;

#[macro_use]
mod macros;

mod gradients;
mod resolve_inherit;
mod fix_attrs;
