// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

use {
    AttributeId,
    Node,
    ValueId,
    WriteBuffer,
    WriteOptions,
    ToStringWithOptions,
};
use types::{
    path,
    Align,
    AspectRatio,
    Color,
    Length,
    LengthList,
    LengthUnit,
    NumberList,
    Points,
    Transform,
    ViewBox,
};

// TODO: custom debug

/// Value of the SVG attribute.
#[derive(Clone,PartialEq,Debug)]
#[allow(missing_docs)]
pub enum AttributeValue {
    AspectRatio(AspectRatio),
    Color(Color),
    /// FuncIRI
    FuncLink(Node),
    Length(Length),
    LengthList(LengthList),
    /// IRI
    Link(Node),
    Number(f64),
    NumberList(NumberList),
    Path(path::Path),
    Points(Points),
    PredefValue(ValueId),
    String(String),
    Transform(Transform),
    ViewBox(ViewBox),
}

macro_rules! impl_from {
    ($vt:ty, $vtn:ident) => (
        impl From<$vt> for AttributeValue {
            fn from(value: $vt) -> AttributeValue {
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
impl_from!(path::Path, Path);
impl_from!(Points, Points);
impl_from!(ValueId, PredefValue);
impl_from!(String, String);
impl_from!(Transform, Transform);
impl_from!(ViewBox, ViewBox);

// TODO: bad, hidden allocation
impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &str) -> AttributeValue {
        AttributeValue::String(value.to_owned())
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> AttributeValue {
        AttributeValue::Number(f64::from(value))
    }
}

impl From<(i32, LengthUnit)> for AttributeValue {
    fn from(value: (i32, LengthUnit)) -> AttributeValue {
        AttributeValue::Length(Length::new(f64::from(value.0), value.1))
    }
}

impl From<(f64, LengthUnit)> for AttributeValue {
    fn from(value: (f64, LengthUnit)) -> AttributeValue {
        AttributeValue::Length(Length::new(value.0, value.1))
    }
}

impl AttributeValue {
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
            | AttributeId::TextRendering => some!(ValueId::Auto),

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
            | AttributeId::TextDecoration => some!(ValueId::None),

              AttributeId::FontStretch
            | AttributeId::FontStyle
            | AttributeId::FontVariant
            | AttributeId::FontWeight
            | AttributeId::LetterSpacing
            | AttributeId::UnicodeBidi
            | AttributeId::WordSpacing => some!(ValueId::Normal),

              AttributeId::Fill
            | AttributeId::FloodColor
            | AttributeId::StopColor => some!(Color::new(0, 0, 0)),

              AttributeId::FillOpacity
            | AttributeId::FloodOpacity
            | AttributeId::Opacity
            | AttributeId::StopOpacity
            | AttributeId::StrokeOpacity => some!(1.0),

              AttributeId::ClipRule
            | AttributeId::FillRule => some!(ValueId::Nonzero),

            AttributeId::BaselineShift =>               some!(ValueId::Baseline),
            AttributeId::ColorInterpolation =>          some!(ValueId::SRGB),
            AttributeId::ColorInterpolationFilters =>   some!(ValueId::LinearRGB),
            AttributeId::Direction =>                   some!(ValueId::Ltr),
            AttributeId::Display =>                     some!(ValueId::Inline),
            AttributeId::EnableBackground =>            some!(ValueId::Accumulate),
            AttributeId::FontSize =>                    some!(ValueId::Medium),
            AttributeId::GlyphOrientationHorizontal =>  some!("0deg"),
            AttributeId::LightingColor =>               some!(Color::new(255, 255, 255)),
            AttributeId::StrokeDashoffset =>            some!((0.0, LengthUnit::None)),
            AttributeId::StrokeLinecap =>               some!(ValueId::Butt),
            AttributeId::StrokeLinejoin =>              some!(ValueId::Miter),
            AttributeId::StrokeMiterlimit =>            some!((4.0, LengthUnit::None)),
            AttributeId::StrokeWidth =>                 some!((1.0, LengthUnit::None)),
            AttributeId::TextAnchor =>                  some!(ValueId::Start),
            AttributeId::Visibility =>                  some!(ValueId::Visible),
            AttributeId::WritingMode =>                 some!(ValueId::LrTb),
            _ => None,
        }
    }
}

impl WriteBuffer for AttributeValue {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        match *self {
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
                n.write_buf_opt(opt, buf);
            }
            AttributeValue::NumberList(ref list) => {
                list.write_buf_opt(opt, buf);
            }
            AttributeValue::Length(ref l) => {
                l.write_buf_opt(opt, buf);
            }
            AttributeValue::LengthList(ref list) => {
                list.write_buf_opt(opt, buf);
            }
            AttributeValue::Transform(ref t) => {
                t.write_buf_opt(opt, buf);
            }
            AttributeValue::Path(ref p) => {
                p.write_buf_opt(opt, buf);
            }
            AttributeValue::Points(ref p) => {
                p.write_buf_opt(opt, buf);
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
            AttributeValue::Color(ref c) => {
                c.write_buf_opt(opt, buf);
            }
            AttributeValue::PredefValue(ref v) => {
                buf.extend_from_slice(v.name().as_bytes())
            }
            AttributeValue::ViewBox(vb) => {
                vb.x.write_buf_opt(opt, buf);
                buf.push(b' ');
                vb.y.write_buf_opt(opt, buf);
                buf.push(b' ');
                vb.w.write_buf_opt(opt, buf);
                buf.push(b' ');
                vb.h.write_buf_opt(opt, buf);
            }
            AttributeValue::AspectRatio(ratio) => {
                if ratio.defer {
                    buf.extend_from_slice(b"defer ");
                }

                let align = match ratio.align {
                    Align::None     => "none",
                    Align::XMinYMin => "xMinYMin",
                    Align::XMidYMin => "xMidYMin",
                    Align::XMaxYMin => "xMaxYMin",
                    Align::XMinYMid => "xMinYMid",
                    Align::XMidYMid => "xMidYMid",
                    Align::XMaxYMid => "xMaxYMid",
                    Align::XMinYMax => "xMinYMax",
                    Align::XMidYMax => "xMidYMax",
                    Align::XMaxYMax => "xMaxYMax",
                };

                buf.extend_from_slice(align.as_bytes());

                if ratio.slice {
                    buf.extend_from_slice(b" slice");
                }
            }
        }
    }
}

impl_display!(AttributeValue);
