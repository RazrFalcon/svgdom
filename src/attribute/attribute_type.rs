// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use {
    Attribute,
    AttributeId,
    Name,
};

/// This trait contains methods that check attribute's type according to the
/// [SVG spec](https://www.w3.org/TR/SVG/intro.html#Definitions).
pub trait AttributeType {
    /// Returns `true` if the current attribute is part of
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    fn is_presentation(&self) -> bool;

    /// Returns `true` if the current attribute is part of inheritable
    /// [presentation attributes](https://www.w3.org/TR/SVG/propidx.html).
    fn is_inheritable(&self) -> bool;

    /// Returns `true` if the current attribute is part of
    /// [animation event attributes](https://www.w3.org/TR/SVG/intro.html#TermAnimationEventAttribute).
    fn is_animation_event(&self) -> bool;

    /// Returns `true` if the current attribute is part of
    /// [graphical event attributes](https://www.w3.org/TR/SVG/intro.html#TermGraphicalEventAttribute).
    fn is_graphical_event(&self) -> bool;

    /// Returns `true` if the current attribute is part of
    /// [document event attributes](https://www.w3.org/TR/SVG/intro.html#TermDocumentEventAttribute).
    fn is_document_event(&self) -> bool;

    /// Returns `true` if the current attribute is part of
    /// [conditional processing attributes
    /// ](https://www.w3.org/TR/SVG/intro.html#TermConditionalProcessingAttribute).
    fn is_conditional_processing(&self) -> bool;

    /// Returns `true` if the current attribute is part of
    /// [core attributes](https://www.w3.org/TR/SVG/intro.html#TermCoreAttributes).
    ///
    /// **NOTE:** the `id` attribute is part of core attributes, but we don't store
    /// it in `Attributes` since it's part of the `Node` struct.
    fn is_core(&self) -> bool;

    /// Returns `true` if the current attribute is part of fill attributes.
    ///
    /// List of fill attributes: `fill`, `fill-opacity`, `fill-rule`.
    ///
    /// This check is not defined by the SVG spec.
    fn is_fill(&self) -> bool;

    /// Returns `true` if the current attribute is part of stroke attributes.
    ///
    /// List of stroke attributes: `stroke`, `stroke-dasharray`, `stroke-dashoffset`,
    /// `stroke-dashoffset`, `stroke-linecap`, `stroke-linejoin`, `stroke-miterlimit`,
    /// `stroke-opacity`, `stroke-width`.
    ///
    /// This check is not defined by the SVG spec.
    fn is_stroke(&self) -> bool;
}

macro_rules! is_func {
    ($name:ident, $($pattern:tt)+) => (
        fn $name(&self) -> bool {
            if let Some(id) = self.id() {
                match id {
                    $($pattern)+ => true,
                    _ => false
                }
            } else {
                false
            }
        }
    )
}

impl AttributeType for Attribute {
    is_func!(is_presentation,
          AttributeId::AlignmentBaseline
        | AttributeId::BaselineShift
        | AttributeId::Clip
        | AttributeId::ClipPath
        | AttributeId::ClipRule
        | AttributeId::Color
        | AttributeId::ColorInterpolation
        | AttributeId::ColorInterpolationFilters
        | AttributeId::ColorProfile
        | AttributeId::ColorRendering
        | AttributeId::Cursor
        | AttributeId::Direction
        | AttributeId::Display
        | AttributeId::DominantBaseline
        | AttributeId::EnableBackground
        | AttributeId::Fill
        | AttributeId::FillOpacity
        | AttributeId::FillRule
        | AttributeId::Filter
        | AttributeId::FloodColor
        | AttributeId::FloodOpacity
        | AttributeId::Font
        | AttributeId::FontFamily
        | AttributeId::FontSize
        | AttributeId::FontSizeAdjust
        | AttributeId::FontStretch
        | AttributeId::FontStyle
        | AttributeId::FontVariant
        | AttributeId::FontWeight
        | AttributeId::GlyphOrientationHorizontal
        | AttributeId::GlyphOrientationVertical
        | AttributeId::ImageRendering
        | AttributeId::Kerning
        | AttributeId::LetterSpacing
        | AttributeId::LightingColor
        | AttributeId::Marker
        | AttributeId::MarkerEnd
        | AttributeId::MarkerMid
        | AttributeId::MarkerStart
        | AttributeId::Mask
        | AttributeId::Opacity
        | AttributeId::Overflow
        | AttributeId::PointerEvents
        | AttributeId::ShapeRendering
        | AttributeId::StopColor
        | AttributeId::StopOpacity
        | AttributeId::Stroke
        | AttributeId::StrokeDasharray
        | AttributeId::StrokeDashoffset
        | AttributeId::StrokeLinecap
        | AttributeId::StrokeLinejoin
        | AttributeId::StrokeMiterlimit
        | AttributeId::StrokeOpacity
        | AttributeId::StrokeWidth
        | AttributeId::TextAnchor
        | AttributeId::TextDecoration
        | AttributeId::TextRendering
        | AttributeId::UnicodeBidi
        | AttributeId::Visibility
        | AttributeId::WordSpacing
        | AttributeId::WritingMode);

    fn is_inheritable(&self) -> bool {
        if self.is_presentation() {
            match self.name {
                Name::Id(id) => !is_non_inheritable(id),
                Name::Name(_) => false,
            }
        } else {
            false
        }
    }

    is_func!(is_animation_event,
          AttributeId::Onbegin
        | AttributeId::Onend
        | AttributeId::Onload
        | AttributeId::Onrepeat);

    is_func!(is_graphical_event,
          AttributeId::Onactivate
        | AttributeId::Onclick
        | AttributeId::Onfocusin
        | AttributeId::Onfocusout
        | AttributeId::Onload
        | AttributeId::Onmousedown
        | AttributeId::Onmousemove
        | AttributeId::Onmouseout
        | AttributeId::Onmouseover
        | AttributeId::Onmouseup);

    is_func!(is_document_event,
          AttributeId::Onabort
        | AttributeId::Onerror
        | AttributeId::Onresize
        | AttributeId::Onscroll
        | AttributeId::Onunload
        | AttributeId::Onzoom);

    is_func!(is_conditional_processing,
          AttributeId::RequiredExtensions
        | AttributeId::RequiredFeatures
        | AttributeId::SystemLanguage);

    is_func!(is_core,
          AttributeId::XmlBase
        | AttributeId::XmlLang
        | AttributeId::XmlSpace);

    is_func!(is_fill,
          AttributeId::Fill
        | AttributeId::FillOpacity
        | AttributeId::FillRule);

    is_func!(is_stroke,
          AttributeId::Stroke
        | AttributeId::StrokeDasharray
        | AttributeId::StrokeDashoffset
        | AttributeId::StrokeLinecap
        | AttributeId::StrokeLinejoin
        | AttributeId::StrokeMiterlimit
        | AttributeId::StrokeOpacity
        | AttributeId::StrokeWidth);
}

// NOTE: `visibility` is marked as inheritable here: https://www.w3.org/TR/SVG/propidx.html,
// but here https://www.w3.org/TR/SVG/painting.html#VisibilityControl
// we have "Note that `visibility` is not an inheritable property."
// And according to webkit, it's really non-inheritable.
fn is_non_inheritable(id: AttributeId) -> bool {
    match id {
          AttributeId::AlignmentBaseline
        | AttributeId::BaselineShift
        | AttributeId::Clip
        | AttributeId::ClipPath
        | AttributeId::Display
        | AttributeId::DominantBaseline
        | AttributeId::EnableBackground
        | AttributeId::Filter
        | AttributeId::FloodColor
        | AttributeId::FloodOpacity
        | AttributeId::LightingColor
        | AttributeId::Mask
        | AttributeId::Opacity
        | AttributeId::Overflow
        | AttributeId::StopColor
        | AttributeId::StopOpacity
        | AttributeId::TextDecoration
        | AttributeId::UnicodeBidi
        | AttributeId::Visibility => true,
        _ => false
    }
}
