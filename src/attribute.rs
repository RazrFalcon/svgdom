// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use super::{
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
    LengthUnit,
    Transform
};
use types::path;

static PRESENTATION_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::AlignmentBaseline,
    AttributeId::BaselineShift,
    AttributeId::Clip,
    AttributeId::ClipPath,
    AttributeId::ClipRule,
    AttributeId::Color,
    AttributeId::ColorInterpolation,
    AttributeId::ColorInterpolationFilters,
    AttributeId::ColorProfile,
    AttributeId::ColorRendering,
    AttributeId::Cursor,
    AttributeId::Direction,
    AttributeId::Display,
    AttributeId::DominantBaseline,
    AttributeId::EnableBackground,
    AttributeId::Fill,
    AttributeId::FillOpacity,
    AttributeId::FillRule,
    AttributeId::Filter,
    AttributeId::FloodColor,
    AttributeId::FloodOpacity,
    AttributeId::Font,
    AttributeId::FontFamily,
    AttributeId::FontSize,
    AttributeId::FontSizeAdjust,
    AttributeId::FontStretch,
    AttributeId::FontStyle,
    AttributeId::FontVariant,
    AttributeId::FontWeight,
    AttributeId::GlyphOrientationHorizontal,
    AttributeId::GlyphOrientationVertical,
    AttributeId::ImageRendering,
    AttributeId::Kerning,
    AttributeId::LetterSpacing,
    AttributeId::LightingColor,
    AttributeId::Marker,
    AttributeId::MarkerEnd,
    AttributeId::MarkerMid,
    AttributeId::MarkerStart,
    AttributeId::Mask,
    AttributeId::Opacity,
    AttributeId::Overflow,
    AttributeId::PointerEvents,
    AttributeId::ShapeRendering,
    AttributeId::StopColor,
    AttributeId::StopOpacity,
    AttributeId::Stroke,
    AttributeId::StrokeDasharray,
    AttributeId::StrokeDashoffset,
    AttributeId::StrokeLinecap,
    AttributeId::StrokeLinejoin,
    AttributeId::StrokeMiterlimit,
    AttributeId::StrokeOpacity,
    AttributeId::StrokeWidth,
    AttributeId::TextAnchor,
    AttributeId::TextDecoration,
    AttributeId::TextRendering,
    AttributeId::UnicodeBidi,
    AttributeId::Visibility,
    AttributeId::WordSpacing,
    AttributeId::WritingMode,
];

static NON_INHERITABLE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::AlignmentBaseline,
    AttributeId::BaselineShift,
    AttributeId::Clip,
    AttributeId::ClipPath,
    AttributeId::Display,
    AttributeId::DominantBaseline,
    AttributeId::EnableBackground,
    AttributeId::Filter,
    AttributeId::FloodColor,
    AttributeId::FloodOpacity,
    AttributeId::LightingColor,
    AttributeId::Mask,
    AttributeId::Opacity,
    AttributeId::Overflow,
    AttributeId::StopColor,
    AttributeId::StopOpacity,
    AttributeId::DominantBaseline,
    AttributeId::TextDecoration,
    AttributeId::UnicodeBidi,
];

static ANIMATION_EVENT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Onbegin,
    AttributeId::Onend,
    AttributeId::Onload,
    AttributeId::Onrepeat,
];

static GRAPHICAL_EVENT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Onactivate,
    AttributeId::Onclick,
    AttributeId::Onfocusin,
    AttributeId::Onfocusout,
    AttributeId::Onload,
    AttributeId::Onmousedown,
    AttributeId::Onmousemove,
    AttributeId::Onmouseout,
    AttributeId::Onmouseover,
    AttributeId::Onmouseup,
];

static DOCUMENT_EVENT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Onabort,
    AttributeId::Onerror,
    AttributeId::Onresize,
    AttributeId::Onscroll,
    AttributeId::Onunload,
    AttributeId::Onzoom,
];

static CONDITIONAL_PROCESSING_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::RequiredExtensions,
    AttributeId::RequiredFeatures,
    AttributeId::SystemLanguage,
];

static CORE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::XmlBase,
    AttributeId::XmlLang,
    AttributeId::XmlSpace,
];

static FILL_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Fill,
    AttributeId::FillOpacity,
    AttributeId::FillRule,
];

static STROKE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Stroke,
    AttributeId::StrokeDasharray,
    AttributeId::StrokeDashoffset,
    AttributeId::StrokeLinecap,
    AttributeId::StrokeLinejoin,
    AttributeId::StrokeMiterlimit,
    AttributeId::StrokeOpacity,
    AttributeId::StrokeWidth,
];

/// Representation of the `<list-of-numbers>`.
pub type NumberList = Vec<f64>;
/// Representation of the `<list-of-lengths>`.
pub type LengthList = Vec<Length>;

/// Value of SVG attribute.
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

impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &str) -> AttributeValue {
        AttributeValue::String(value.to_owned())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> AttributeValue {
        AttributeValue::String(value)
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> AttributeValue {
        AttributeValue::Number(value as f64)
    }
}

impl From<f64> for AttributeValue {
    fn from(value: f64) -> AttributeValue {
        AttributeValue::Number(value)
    }
}

impl From<NumberList> for AttributeValue {
    fn from(value: NumberList) -> AttributeValue {
        AttributeValue::NumberList(value)
    }
}

impl From<Length> for AttributeValue {
    fn from(value: Length) -> AttributeValue {
        AttributeValue::Length(value)
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

impl From<LengthList> for AttributeValue {
    fn from(value: LengthList) -> AttributeValue {
        AttributeValue::LengthList(value)
    }
}

impl From<Transform> for AttributeValue {
    fn from(value: Transform) -> AttributeValue {
        AttributeValue::Transform(value)
    }
}

impl From<path::Path> for AttributeValue {
    fn from(value: path::Path) -> AttributeValue {
        AttributeValue::Path(value)
    }
}

impl From<Color> for AttributeValue {
    fn from(value: Color) -> AttributeValue {
        AttributeValue::Color(value)
    }
}

impl From<ValueId> for AttributeValue {
    fn from(value: ValueId) -> AttributeValue {
        AttributeValue::PredefValue(value)
    }
}

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

    /// Constructs a new attribute value with default value, if it's known.
    pub fn default_value(id: AttributeId) -> Option<AttributeValue> {
        macro_rules! some {
            ($expr:expr) => (Some(AttributeValue::from($expr)))
        }

        match id {
            AttributeId::AlignmentBaseline =>           some!(ValueId::Auto),
            AttributeId::BaselineShift =>               some!(ValueId::Baseline),
            AttributeId::ClipPath =>                    some!(ValueId::None),
            AttributeId::ClipRule =>                    some!(ValueId::Nonzero),
            AttributeId::ColorInterpolation =>          some!(ValueId::SRGB),
            AttributeId::ColorInterpolationFilters =>   some!(ValueId::LinearRGB),
            AttributeId::ColorProfile =>                some!(ValueId::Auto),
            AttributeId::ColorRendering =>              some!(ValueId::Auto),
            AttributeId::Cursor =>                      some!(ValueId::Auto),
            AttributeId::Direction =>                   some!(ValueId::Ltr),
            AttributeId::Display =>                     some!(ValueId::Inline),
            AttributeId::DominantBaseline =>            some!(ValueId::Auto),
            AttributeId::EnableBackground =>            some!(ValueId::Accumulate),
            AttributeId::Fill =>                        some!(Color::new(0, 0, 0)),
            AttributeId::FillOpacity =>                 some!(1.0),
            AttributeId::FillRule =>                    some!(ValueId::Nonzero),
            AttributeId::Filter =>                      some!(ValueId::None),
            AttributeId::FloodColor =>                  some!(Color::new(0, 0, 0)),
            AttributeId::FloodOpacity =>                some!(1.0),
            AttributeId::FontSizeAdjust =>              some!(ValueId::None),
            AttributeId::FontSize =>                    some!(ValueId::Medium),
            AttributeId::FontStretch =>                 some!(ValueId::Normal),
            AttributeId::FontStyle =>                   some!(ValueId::Normal),
            AttributeId::FontVariant =>                 some!(ValueId::Normal),
            AttributeId::FontWeight =>                  some!(ValueId::Normal),
            AttributeId::GlyphOrientationHorizontal =>  some!("0deg"),
            AttributeId::GlyphOrientationVertical =>    some!(ValueId::Auto),
            AttributeId::ImageRendering =>              some!(ValueId::Auto),
            AttributeId::Kerning =>                     some!(ValueId::Auto),
            AttributeId::LetterSpacing =>               some!(ValueId::Normal),
            AttributeId::LightingColor =>               some!(Color::new(255, 255, 255)),
            AttributeId::Marker =>                      some!(ValueId::None),
            AttributeId::MarkerStart =>                 some!(ValueId::None),
            AttributeId::MarkerMid =>                   some!(ValueId::None),
            AttributeId::MarkerEnd =>                   some!(ValueId::None),
            AttributeId::Mask =>                        some!(ValueId::None),
            AttributeId::Opacity =>                     some!(1.0),
            AttributeId::ShapeRendering =>              some!(ValueId::Auto),
            AttributeId::StopColor =>                   some!(Color::new(0, 0, 0)),
            AttributeId::StopOpacity =>                 some!(1.0),
            AttributeId::Stroke =>                      some!(ValueId::None),
            AttributeId::StrokeDasharray =>             some!(ValueId::None),
            AttributeId::StrokeDashoffset =>            some!((0.0, LengthUnit::None)),
            AttributeId::StrokeLinecap =>               some!(ValueId::Butt),
            AttributeId::StrokeLinejoin =>              some!(ValueId::Miter),
            AttributeId::StrokeMiterlimit =>            some!((4.0, LengthUnit::None)),
            AttributeId::StrokeOpacity =>               some!(1.0),
            AttributeId::StrokeWidth =>                 some!((1.0, LengthUnit::None)),
            AttributeId::TextAnchor =>                  some!(ValueId::Start),
            AttributeId::TextDecoration =>              some!(ValueId::None),
            AttributeId::TextRendering =>               some!(ValueId::Auto),
            AttributeId::UnicodeBidi =>                 some!(ValueId::Normal),
            AttributeId::Visibility =>                  some!(ValueId::Visible),
            AttributeId::WordSpacing =>                 some!(ValueId::Normal),
            AttributeId::WritingMode =>                 some!(ValueId::LrTb),
            _ => None,
        }
    }
}

impl WriteBuffer for AttributeValue {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        // TODO: save stdDeviation with transform precision
        match *self {
            AttributeValue::String(ref s) => {
                buf.extend_from_slice(s.as_bytes());
            },
            AttributeValue::Number(ref n) => {
                n.write_buf_opt(opt, buf);
            },
            AttributeValue::NumberList(ref list) => {
                for (i, num) in list.iter().enumerate() {
                    num.write_buf_opt(opt, buf);

                    if i < list.len() - 1 {
                        buf.push(b' ');
                    }
                }
            },
            AttributeValue::Length(ref l) => {
                l.write_buf_opt(opt, buf);
            },
            AttributeValue::LengthList(ref list) => {
                // TODO: impl for struct
                for (n, l) in list.iter().enumerate() {
                    l.write_buf_opt(opt, buf);
                    if n < list.len() - 1 {
                        buf.push(b' ');
                    }
                }
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

impl_display!(AttributeValue);

/// Representation oh the SVG attribute object.
#[derive(PartialEq,Clone,Debug)]
pub struct Attribute {
    /// Internal ID of the attribute.
    pub id: AttributeId,
    /// Attribute value.
    pub value: AttributeValue,
    /// Visibility.
    ///
    /// Unlike many other DOM implementations, libsvgdom supports hiding of the attributes,
    /// instead removing them. Invisible attributes acts just like other attributes,
    /// but they will not be printed during SVG writing. Unless you enable them via `WriteOptions`.
    ///
    /// All attributes are visible by default.
    pub visible: bool,
}

macro_rules! impl_is_type {
    ($name:ident, $t:ident) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> bool {
            match self.value {
                AttributeValue::$t(_) => true,
                _ => false,
            }
        }
    )
}

impl Attribute {
    /// Constructs a new attribute.
    pub fn new<T>(id: AttributeId, value: T) -> Attribute
        where AttributeValue: From<T>
    {
        Attribute {
            id: id,
            value: AttributeValue::from(value),
            visible: true,
        }
    }

    /// Constructs a new attribute with default value, if it known.
    pub fn default(id: AttributeId) -> Option<Attribute> {
        match AttributeValue::default_value(id) {
            Some(v) => Some(Attribute::new(id, v)),
            None => None,
        }
    }

    /// Returns `true` if current attribute's value is equal to default by SVG spec.
    pub fn check_is_default(&self) -> bool {
        match AttributeValue::default_value(self.id) {
            Some(v) => self.value == v,
            None => false,
        }
    }

    /// Returns `true` if current attribute is part of
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    pub fn is_presentation(&self) -> bool {
        PRESENTATION_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of inheritable
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    pub fn is_inheritable(&self) -> bool {
        self.is_presentation() && NON_INHERITABLE_ATTRIBUTES.binary_search(&self.id).is_err()
    }

    /// Returns `true` if current attribute is part of
    /// [animation event attributes](https://www.w3.org/TR/SVG/intro.html#TermAnimationEventAttribute).
    pub fn is_animation_event(&self) -> bool {
        ANIMATION_EVENT_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of
    /// [graphical event attributes](https://www.w3.org/TR/SVG/intro.html#TermGraphicalEventAttribute).
    pub fn is_graphical_event(&self) -> bool {
        GRAPHICAL_EVENT_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of
    /// [document event attributes](https://www.w3.org/TR/SVG/intro.html#TermDocumentEventAttribute).
    pub fn is_document_event(&self) -> bool {
        DOCUMENT_EVENT_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of
    /// [conditional processing attributes](https://www.w3.org/TR/SVG/intro.html#TermConditionalProcessingAttribute).
    pub fn is_conditional_processing(&self) -> bool {
        CONDITIONAL_PROCESSING_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of
    /// [core attributes](https://www.w3.org/TR/SVG/intro.html#TermCoreAttributes).
    ///
    /// **NOTE:** the `id` attribute is part of core attributes, but we don't store it here
    /// since it's part of the `Node` struct.
    pub fn is_core(&self) -> bool {
        CORE_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of fill attributes.
    ///
    /// List of fill attributes: `fill`, `fill-opacity`, `fill-rule`.
    ///
    /// This check is not defined by SVG spec.
    pub fn is_fill(&self) -> bool {
        FILL_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    /// Returns `true` if current attribute is part of stroke attributes.
    ///
    /// List of stroke attributes: `stroke`, `stroke-dasharray`, `stroke-dashoffset`,
    /// `stroke-dashoffset`, `stroke-linecap`, `stroke-linejoin`, `stroke-miterlimit`,
    /// `stroke-opacity`, `stroke-width`.
    ///
    /// This check is not defined by SVG spec.
    pub fn is_stroke(&self) -> bool {
        STROKE_ATTRIBUTES.binary_search(&self.id).is_ok()
    }

    impl_is_type!(is_color, Color);
    impl_is_type!(is_length, Length);
    impl_is_type!(is_length_list, LengthList);
    impl_is_type!(is_link, Link);
    impl_is_type!(is_func_link, FuncLink);
    impl_is_type!(is_number, Number);
    impl_is_type!(is_number_list, NumberList);
    impl_is_type!(is_path, Path);
    impl_is_type!(is_predef_value, PredefValue);
    impl_is_type!(is_string, String);
    impl_is_type!(is_transform, Transform);
}

fn write_quote(opt: &WriteOptions, out: &mut Vec<u8>) {
    if opt.use_single_quote {
        out.push(b'\'');
    } else {
        out.push(b'"');
    }
}

impl WriteBuffer for Attribute {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        let name = self.id.name();

        buf.extend_from_slice(name.as_bytes());
        buf.push(b'=');
        write_quote(opt, buf);
        self.value.write_buf_opt(opt, buf);
        write_quote(opt, buf);
    }
}

impl_display!(Attribute);
