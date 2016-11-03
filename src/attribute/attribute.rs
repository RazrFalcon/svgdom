// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use {
    AttributeId,
    AttributeValue,
    Name,
    NameRef,
    SvgId,
    WriteBuffer,
    WriteOptions,
    WriteToString,
};

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

// NOTE: `visibility` is marked as inheritable here: https://www.w3.org/TR/SVG/propidx.html,
// but here https://www.w3.org/TR/SVG/painting.html#VisibilityControl
// we have "Note that `visibility` is not an inheritable property."
// And according to webkit, it's really non-inheritable.
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
    AttributeId::Visibility,
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

/// Type alias for `Name<AttributeId>`.
pub type AttributeName = Name<AttributeId>;
/// Type alias for `NameRef<AttributeId>`.
pub type AttributeNameRef<'a> = NameRef<'a, AttributeId>;

impl SvgId for AttributeId {
    fn name(&self) -> &str { self.name() }
}

/// Representation of the SVG attribute object.
#[derive(PartialEq,Clone,Debug)]
pub struct Attribute {
    /// Attribute name.
    pub name: AttributeName,
    /// Attribute value.
    pub value: AttributeValue,
    /// Visibility.
    ///
    /// Unlike many other DOM implementations, libsvgdom supports hiding of the attributes,
    /// instead of removing them. Invisible attributes act just like other attributes,
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
    pub fn new<'a, N, T>(name: N, value: T) -> Attribute
        where AttributeNameRef<'a>: From<N>, AttributeValue: From<T>
    {
        let n = AttributeNameRef::from(name);
        Attribute {
            name: AttributeName::from(n),
            value: AttributeValue::from(value),
            visible: true,
        }
    }

    /// Returns an SVG attribute ID.
    pub fn id(&self) -> Option<AttributeId> {
        match self.name {
            Name::Id(id) => Some(id),
            Name::Name(_) => None,
        }
    }

    /// Returns `true` if the attribute has the selected ID.
    pub fn has_id(&self, id: AttributeId) -> bool {
        match self.name {
            Name::Id(id2) => id2 == id,
            Name::Name(_) => false,
        }
    }

    /// Returns `true` if the attribute is an SVG attribute.
    pub fn is_svg(&self) -> bool {
        match self.name {
            Name::Id(_) => true,
            Name::Name(_) => false,
        }
    }

    /// Constructs a new attribute with a default value, if it known.
    pub fn default(id: AttributeId) -> Option<Attribute> {
        match AttributeValue::default_value(id) {
            Some(v) => Some(Attribute::new(id, v)),
            None => None,
        }
    }

    /// Returns `true` if the current attribute's value is equal to a default by the SVG spec.
    pub fn check_is_default(&self) -> bool {
        if let Name::Id(id) = self.name {
            match AttributeValue::default_value(id) {
                Some(v) => self.value == v,
                None => false,
            }
        } else {
            false
        }
    }

    /// Returns `true` if the current attribute is part of
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    pub fn is_presentation(&self) -> bool {
        self.list_contains(PRESENTATION_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of inheritable
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    pub fn is_inheritable(&self) -> bool {
        if self.is_presentation() {
            match self.name {
                Name::Id(id) => NON_INHERITABLE_ATTRIBUTES.binary_search(&id).is_err(),
                Name::Name(_) => false,
            }
        } else {
            false
        }
    }

    /// Returns `true` if the current attribute is part of
    /// [animation event attributes](https://www.w3.org/TR/SVG/intro.html#TermAnimationEventAttribute).
    pub fn is_animation_event(&self) -> bool {
        self.list_contains(ANIMATION_EVENT_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of
    /// [graphical event attributes](https://www.w3.org/TR/SVG/intro.html#TermGraphicalEventAttribute).
    pub fn is_graphical_event(&self) -> bool {
        self.list_contains(GRAPHICAL_EVENT_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of
    /// [document event attributes](https://www.w3.org/TR/SVG/intro.html#TermDocumentEventAttribute).
    pub fn is_document_event(&self) -> bool {
        self.list_contains(DOCUMENT_EVENT_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of
    /// [conditional processing attributes
    /// ](https://www.w3.org/TR/SVG/intro.html#TermConditionalProcessingAttribute).
    pub fn is_conditional_processing(&self) -> bool {
        self.list_contains(CONDITIONAL_PROCESSING_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of
    /// [core attributes](https://www.w3.org/TR/SVG/intro.html#TermCoreAttributes).
    ///
    /// **NOTE:** the `id` attribute is part of core attributes, but we don't store it here
    /// since it's part of the `Node` struct.
    pub fn is_core(&self) -> bool {
        self.list_contains(CORE_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of fill attributes.
    ///
    /// List of fill attributes: `fill`, `fill-opacity`, `fill-rule`.
    ///
    /// This check is not defined by the SVG spec.
    pub fn is_fill(&self) -> bool {
        self.list_contains(FILL_ATTRIBUTES)
    }

    /// Returns `true` if the current attribute is part of stroke attributes.
    ///
    /// List of stroke attributes: `stroke`, `stroke-dasharray`, `stroke-dashoffset`,
    /// `stroke-dashoffset`, `stroke-linecap`, `stroke-linejoin`, `stroke-miterlimit`,
    /// `stroke-opacity`, `stroke-width`.
    ///
    /// This check is not defined by the SVG spec.
    pub fn is_stroke(&self) -> bool {
        self.list_contains(STROKE_ATTRIBUTES)
    }

    fn list_contains(&self, list: &[AttributeId]) -> bool {
        match self.name {
            Name::Id(id) => list.binary_search(&id).is_ok(),
            Name::Name(_) => false,
        }
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
    out.push(if opt.use_single_quote { b'\'' } else { b'"' });
}

impl WriteBuffer for Attribute {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        match self.name {
            Name::Id(id) => buf.extend_from_slice(id.name().as_bytes()),
            Name::Name(ref name) => buf.extend_from_slice(name.as_bytes()),
        }
        buf.push(b'=');
        write_quote(opt, buf);
        self.value.write_buf_opt(opt, buf);
        write_quote(opt, buf);
    }
}

impl_display!(Attribute);
