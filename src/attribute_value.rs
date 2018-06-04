// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

use {
    AspectRatio,
    AttributeId,
    Color,
    Length,
    LengthList,
    LengthUnit,
    Node,
    NumberList,
    PaintFallback,
    Path,
    Points,
    Transform,
    ValueWriteBuffer,
    ViewBox,
    WriteBuffer,
    WriteOptions,
};

// TODO: custom debug

/// Value of the SVG attribute.
#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum AttributeValue {
    None,
    Inherit,
    CurrentColor,
    AspectRatio(AspectRatio),
    Color(Color),
    /// FuncIRI
    FuncLink(Node),
    Paint(Node, Option<PaintFallback>),
    Length(Length),
    LengthList(LengthList),
    /// IRI
    Link(Node),
    Number(f64),
    NumberList(NumberList),
    Path(Path),
    Points(Points),
    Transform(Transform),
    ViewBox(ViewBox),
    String(String),
}

macro_rules! impl_from {
    ($vt:ty, $vtn:ident) => (
        impl From<$vt> for AttributeValue {
            fn from(value: $vt) -> Self {
                AttributeValue::$vtn(value)
            }
        }
    )
}

impl_from!(AspectRatio, AspectRatio);
impl_from!(Color, Color);
impl_from!(Length, Length);
impl_from!(LengthList, LengthList);
impl_from!(f64, Number);
impl_from!(NumberList, NumberList);
impl_from!(Path, Path);
impl_from!(Points, Points);
impl_from!(String, String);
impl_from!(Transform, Transform);
impl_from!(ViewBox, ViewBox);

// TODO: bad, hidden allocation
impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &str) -> Self {
        AttributeValue::String(value.to_owned())
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> Self {
        AttributeValue::Number(f64::from(value))
    }
}

impl From<(i32, LengthUnit)> for AttributeValue {
    fn from(value: (i32, LengthUnit)) -> Self {
        AttributeValue::Length(Length::new(f64::from(value.0), value.1))
    }
}

impl From<(f64, LengthUnit)> for AttributeValue {
    fn from(value: (f64, LengthUnit)) -> Self {
        AttributeValue::Length(Length::new(value.0, value.1))
    }
}

impl From<PaintFallback> for AttributeValue {
    fn from(value: PaintFallback) -> Self {
        match value {
            PaintFallback::None => AttributeValue::None,
            PaintFallback::CurrentColor => AttributeValue::CurrentColor,
            PaintFallback::Color(c) => AttributeValue::Color(c),
        }
    }
}

// TODO: fix docs
macro_rules! impl_is_type {
    ($name:ident, $t:ident) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> bool {
            match *self {
                AttributeValue::$t(..) => true,
                _ => false,
            }
        }
    )
}

macro_rules! impl_is_type_without_value {
    ($name:ident, $t:ident) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> bool {
            match *self {
                AttributeValue::$t => true,
                _ => false,
            }
        }
    )
}

impl AttributeValue {
    impl_is_type_without_value!(is_none, None);
    impl_is_type_without_value!(is_inherit, Inherit);
    impl_is_type_without_value!(is_current_color, CurrentColor);
    impl_is_type!(is_aspect_ratio, AspectRatio);
    impl_is_type!(is_color, Color);
    impl_is_type!(is_length, Length);
    impl_is_type!(is_length_list, LengthList);
    impl_is_type!(is_link, Link);
    impl_is_type!(is_func_link, FuncLink);
    impl_is_type!(is_paint, Paint);
    impl_is_type!(is_number, Number);
    impl_is_type!(is_number_list, NumberList);
    impl_is_type!(is_path, Path);
    impl_is_type!(is_points, Points);
    impl_is_type!(is_string, String);
    impl_is_type!(is_transform, Transform);
    impl_is_type!(is_viewbox, ViewBox);

    /// Checks that the current attribute value contains a `Node`.
    ///
    /// E.g. `Link`, `FuncLink` and `Paint`.
    pub fn is_link_container(&self) -> bool {
        match *self {
              AttributeValue::Link(_)
            | AttributeValue::FuncLink(_)
            | AttributeValue::Paint(_, _) => true,
            _ => false,
        }
    }

    /// Constructs a new attribute value with a default value, if it's known.
    pub fn default_value(id: AttributeId) -> Option<AttributeValue> {
        macro_rules! some {
            ($expr:expr) => (Some(AttributeValue::from($expr)))
        }

        match id {
              AttributeId::AlignmentBaseline
            | AttributeId::Clip
            | AttributeId::ColorProfile
            | AttributeId::ColorRendering
            | AttributeId::Cursor
            | AttributeId::DominantBaseline
            | AttributeId::GlyphOrientationVertical
            | AttributeId::ImageRendering
            | AttributeId::Kerning
            | AttributeId::ShapeRendering
            | AttributeId::TextRendering => some!("auto"),

              AttributeId::ClipPath
            | AttributeId::Filter
            | AttributeId::FontSizeAdjust
            | AttributeId::Marker
            | AttributeId::MarkerEnd
            | AttributeId::MarkerMid
            | AttributeId::MarkerStart
            | AttributeId::Mask
            | AttributeId::Stroke
            | AttributeId::StrokeDasharray
            | AttributeId::TextDecoration => Some(AttributeValue::None),

              AttributeId::FontStretch
            | AttributeId::FontStyle
            | AttributeId::FontVariant
            | AttributeId::FontWeight
            | AttributeId::LetterSpacing
            | AttributeId::UnicodeBidi
            | AttributeId::WordSpacing => some!("normal"),

              AttributeId::Fill
            | AttributeId::FloodColor
            | AttributeId::StopColor => some!(Color::black()),

              AttributeId::FillOpacity
            | AttributeId::FloodOpacity
            | AttributeId::Opacity
            | AttributeId::StopOpacity
            | AttributeId::StrokeOpacity => some!(1.0),

              AttributeId::ClipRule
            | AttributeId::FillRule => some!("nonzero"),

            AttributeId::BaselineShift =>               some!("baseline"),
            AttributeId::ColorInterpolation =>          some!("sRGB"),
            AttributeId::ColorInterpolationFilters =>   some!("linearRGB"),
            AttributeId::Direction =>                   some!("ltr"),
            AttributeId::Display =>                     some!("inline"),
            AttributeId::EnableBackground =>            some!("accumulate"),
            AttributeId::FontSize =>                    some!("medium"),
            AttributeId::GlyphOrientationHorizontal =>  some!("0deg"),
            AttributeId::LightingColor =>               some!(Color::white()),
            AttributeId::StrokeDashoffset =>            some!((0.0, LengthUnit::None)),
            AttributeId::StrokeLinecap =>               some!("butt"),
            AttributeId::StrokeLinejoin =>              some!("miter"),
            AttributeId::StrokeMiterlimit =>            some!((4.0, LengthUnit::None)),
            AttributeId::StrokeWidth =>                 some!((1.0, LengthUnit::None)),
            AttributeId::TextAnchor =>                  some!("start"),
            AttributeId::Visibility =>                  some!("visible"),
            AttributeId::WritingMode =>                 some!("lr-tb"),
            _ => None,
        }
    }
}

impl WriteBuffer for AttributeValue {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        match *self {
            AttributeValue::None => {
                buf.extend_from_slice(b"none");
            }
            AttributeValue::Inherit => {
                buf.extend_from_slice(b"inherit");
            }
            AttributeValue::CurrentColor => {
                buf.extend_from_slice(b"currentColor");
            }
            AttributeValue::String(ref s) => {
                for c in s.as_bytes() {
                    match *c {
                        b'"' if !opt.use_single_quote => buf.extend_from_slice(b"&quot;"),
                        b'\'' if opt.use_single_quote => buf.extend_from_slice(b"&apos;"),
                        _ => buf.push(*c),
                    }
                }
            }
            AttributeValue::Number(ref n) => {
                n.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::NumberList(ref list) => {
                list.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::Length(ref l) => {
                l.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::LengthList(ref list) => {
                list.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::Transform(ref t) => {
                t.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::Path(ref p) => {
                p.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::Points(ref p) => {
                p.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::Link(ref n) => {
                buf.push(b'#');
                buf.extend_from_slice(n.id().as_bytes());
            }
            AttributeValue::FuncLink(ref n) => {
                buf.extend_from_slice(b"url(#");
                buf.extend_from_slice(n.id().as_bytes());
                buf.push(b')');
            }
            AttributeValue::Paint(ref n, ref fallback) => {
                buf.extend_from_slice(b"url(#");
                buf.extend_from_slice(n.id().as_bytes());
                buf.push(b')');

                if let Some(fallback) = *fallback {
                    buf.push(b' ');
                    match fallback {
                        PaintFallback::None => buf.extend_from_slice(b"none"),
                        PaintFallback::CurrentColor => buf.extend_from_slice(b"currentColor"),
                        PaintFallback::Color(ref c) => c.write_buf_opt(&opt.values, buf),
                    }
                }
            }
            AttributeValue::Color(ref c) => {
                c.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::ViewBox(vb) => {
                vb.write_buf_opt(&opt.values, buf);
            }
            AttributeValue::AspectRatio(ratio) => {
                ratio.write_buf_opt(&opt.values, buf);
            }
        }
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.with_write_opt(&WriteOptions::default()))
    }
}
