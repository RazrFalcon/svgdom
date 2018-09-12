// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    AttributeId,
    AttributeQNameRef,
    ElementId,
    QNameRef,
};

static SVG_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Width),
    QNameRef::Id(AttributeId::Height),
    QNameRef::Id(AttributeId::ViewBox),
    QNameRef::Id(AttributeId::PreserveAspectRatio),
    QNameRef::Id(AttributeId::Version),
    QNameRef::Id(AttributeId::BaseProfile),
];

static RECT_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Width),
    QNameRef::Id(AttributeId::Height),
    QNameRef::Id(AttributeId::Rx),
    QNameRef::Id(AttributeId::Ry),
];

static CIRCLE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::Cx),
    QNameRef::Id(AttributeId::Cy),
    QNameRef::Id(AttributeId::R),
];

static ELLIPSE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::Cx),
    QNameRef::Id(AttributeId::Cy),
    QNameRef::Id(AttributeId::Rx),
    QNameRef::Id(AttributeId::Ry),
];

static LINE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::X1),
    QNameRef::Id(AttributeId::Y1),
    QNameRef::Id(AttributeId::X2),
    QNameRef::Id(AttributeId::Y2),
];

static POLYLINE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::Points),
];

static PATH_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::D),
];

static USE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Width),
    QNameRef::Id(AttributeId::Height),
    QNameRef::Id(AttributeId::Href),
];

static IMAGE_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::PreserveAspectRatio),
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Width),
    QNameRef::Id(AttributeId::Height),
    QNameRef::Id(AttributeId::Href),
];

static TEXT_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Transform),
    QNameRef::Id    (AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Dx),
    QNameRef::Id(AttributeId::Dy),
    QNameRef::Id(AttributeId::Rotate),
];

static TSPAN_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Dx),
    QNameRef::Id(AttributeId::Dy),
    QNameRef::Id(AttributeId::Rotate),
];

static LINEAR_GRADIENT_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::X1),
    QNameRef::Id(AttributeId::Y1),
    QNameRef::Id(AttributeId::X2),
    QNameRef::Id(AttributeId::Y2),
    QNameRef::Id(AttributeId::GradientUnits),
    QNameRef::Id(AttributeId::GradientTransform),
    QNameRef::Id(AttributeId::SpreadMethod),
    QNameRef::Id(AttributeId::Href),
];

static RADIAL_GRADIENT_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::Cx),
    QNameRef::Id(AttributeId::Cy),
    QNameRef::Id(AttributeId::R),
    QNameRef::Id(AttributeId::Fx),
    QNameRef::Id(AttributeId::Fy),
    QNameRef::Id(AttributeId::GradientUnits),
    QNameRef::Id(AttributeId::GradientTransform),
    QNameRef::Id(AttributeId::SpreadMethod),
    QNameRef::Id(AttributeId::Href),
];

static PATTERN_ATTRIBUTES: &'static [AttributeQNameRef<'static>] = &[
    QNameRef::Id(AttributeId::ViewBox),
    QNameRef::Id(AttributeId::X),
    QNameRef::Id(AttributeId::Y),
    QNameRef::Id(AttributeId::Width),
    QNameRef::Id(AttributeId::Height),
    QNameRef::Id(AttributeId::PatternUnits),
    QNameRef::Id(AttributeId::PatternContentUnits),
    QNameRef::Id(AttributeId::PatternTransform),
    QNameRef::Id(AttributeId::Href),
];

pub fn attrs_order_by_element(eid: ElementId) -> &'static [AttributeQNameRef<'static>] {
    match eid {
        ElementId::Svg => SVG_ATTRIBUTES,
        ElementId::Rect => RECT_ATTRIBUTES,
        ElementId::Circle => CIRCLE_ATTRIBUTES,
        ElementId::Ellipse => ELLIPSE_ATTRIBUTES,
        ElementId::Line => LINE_ATTRIBUTES,
        ElementId::Polyline | ElementId::Polygon => POLYLINE_ATTRIBUTES,
        ElementId::Path => PATH_ATTRIBUTES,
        ElementId::Use => USE_ATTRIBUTES,
        ElementId::Image => IMAGE_ATTRIBUTES,
        ElementId::Text => TEXT_ATTRIBUTES,
        ElementId::Tspan => TSPAN_ATTRIBUTES,
        ElementId::LinearGradient => LINEAR_GRADIENT_ATTRIBUTES,
        ElementId::RadialGradient => RADIAL_GRADIENT_ATTRIBUTES,
        ElementId::Pattern => PATTERN_ATTRIBUTES,
        _ => &[],
    }
}
