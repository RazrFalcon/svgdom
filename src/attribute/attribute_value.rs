// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use {
    AttributeId,
    Node,
    ValueId,
    WriteBuffer,
    WriteOptions,
    WriteToString,
};
use types::{
    Color,
    Length,
    LengthList,
    LengthUnit,
    NumberList,
    path,
    Transform,
};

// TODO: custom debug

/// Value of the SVG attribute.
#[derive(Clone,PartialEq,Debug)]
#[allow(missing_docs)]
pub enum AttributeValue {
    Color(Color),
    Length(Length),
    LengthList(LengthList),
    /// IRI
    Link(Node),
    /// FuncIRI
    FuncLink(Node),
    Number(f64),
    NumberList(NumberList),
    Path(path::Path),
    PredefValue(ValueId),
    String(String),
    Transform(Transform),
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

impl_from!(String, String);
impl_from!(f64, Number);
impl_from!(NumberList, NumberList);
impl_from!(Length, Length);
impl_from!(LengthList, LengthList);
impl_from!(Transform, Transform);
impl_from!(Color, Color);
impl_from!(ValueId, PredefValue);
impl_from!(path::Path, Path);

// TODO: bad, hidden allocation
impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &str) -> AttributeValue {
        AttributeValue::String(value.to_owned())
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> AttributeValue {
        AttributeValue::Number(value as f64)
    }
}

impl From<(i32, LengthUnit)> for AttributeValue {
    fn from(value: (i32, LengthUnit)) -> AttributeValue {
        AttributeValue::Length(Length::new(value.0 as f64, value.1))
    }
}

impl From<(f64, LengthUnit)> for AttributeValue {
    fn from(value: (f64, LengthUnit)) -> AttributeValue {
        AttributeValue::Length(Length::new(value.0, value.1))
    }
}

// TODO: maybe remove?
macro_rules! impl_as_type {
    ($name:ident, $t:ident, $out:ty) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> Option<&$out> {
            match *self {
                AttributeValue::$t(ref v) => Some(v),
                _ => None,
            }
        }
    )
}

impl AttributeValue {
    impl_as_type!(as_color, Color, Color);
    impl_as_type!(as_length, Length, Length);
    impl_as_type!(as_length_list, LengthList, LengthList);
    impl_as_type!(as_link, Link, Node);
    impl_as_type!(as_func_link, FuncLink, Node);
    impl_as_type!(as_number, Number, f64);
    impl_as_type!(as_number_list, NumberList, NumberList);
    impl_as_type!(as_path, Path, path::Path);
    impl_as_type!(as_predef_value, PredefValue, ValueId);
    impl_as_type!(as_string, String, String);
    impl_as_type!(as_transform, Transform, Transform);

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

    /// Returns type's name. For the debug purposes.
    pub fn name(&self) -> &str {
        match *self {
            AttributeValue::Color(_) => "Color",
            AttributeValue::Length(_) => "Length",
            AttributeValue::LengthList(_) => "LengthList",
            AttributeValue::Link(_) => "Link",
            AttributeValue::FuncLink(_) => "FuncLink",
            AttributeValue::Number(_) => "Number",
            AttributeValue::NumberList(_) => "NumberList",
            AttributeValue::Path(_) => "Path",
            AttributeValue::PredefValue(_) => "PredefValue",
            AttributeValue::String(_) => "String",
            AttributeValue::Transform(_) => "Transform",
        }
    }
}

impl WriteBuffer for AttributeValue {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        match *self {
            AttributeValue::String(ref s) => {
                buf.extend_from_slice(s.as_bytes());
            },
            AttributeValue::Number(ref n) => {
                n.write_buf_opt(opt, buf);
            },
            AttributeValue::NumberList(ref list) => {
                write_list(list, opt, buf)
            },
            AttributeValue::Length(ref l) => {
                l.write_buf_opt(opt, buf);
            },
            AttributeValue::LengthList(ref list) => {
                write_list(list, opt, buf);
            },
            AttributeValue::Transform(ref t) => {
                t.write_buf_opt(opt, buf);
            }
            AttributeValue::Path(ref p) => {
                p.write_buf_opt(opt, buf);
            }
            AttributeValue::Link(ref n) => {
                buf.push(b'#');
                buf.extend_from_slice(n.id().as_bytes());
            },
            AttributeValue::FuncLink(ref n) => {
                buf.extend_from_slice(b"url(#");
                buf.extend_from_slice(n.id().as_bytes());
                buf.push(b')');
            },
            AttributeValue::Color(ref c) => {
                c.write_buf_opt(opt, buf);
            },
            AttributeValue::PredefValue(ref v) => {
                buf.extend_from_slice(v.name().as_bytes())
            },
        }
    }
}

fn write_list<T: WriteBuffer>(list: &[T], opt: &WriteOptions, buf: &mut Vec<u8>) {
    for (n, l) in list.iter().enumerate() {
        l.write_buf_opt(opt, buf);
        if n < list.len() - 1 {
            buf.push(b' ');
        }
    }
}

impl_display!(AttributeValue);
