use crate::{
    ElementId,
    Node,
};

/// This trait contains methods that check element's type according to the
/// [SVG spec](https://www.w3.org/TR/SVG/intro.html#Definitions).
///
/// Note that all methods works with `Node` type and will return `false`
/// if node's type is not equal to `NodeType::Element`.
///
/// # Panics
///
/// All methods panics if the node is currently mutability borrowed.
pub trait ElementType {
    /// Returns true if the current node is referenced.
    ///
    /// Referenced elements are elements that do not render by itself,
    /// rather defines rendering properties for other.
    ///
    /// List: `altGlyphDef`, `clipPath`, `cursor`, `filter`, `linearGradient`, `marker`,
    /// `mask`, `pattern`, `radialGradient` and `symbol`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/struct.html#Head>
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::{Document, ElementType};
    ///
    /// let doc = Document::from_str(
    ///     "<svg xmlns='http://www.w3.org/2000/svg'><linearGradient/></svg>").unwrap();
    /// let mut iter = doc.root().descendants();
    /// assert_eq!(iter.next().unwrap().is_referenced(), false); // root
    /// assert_eq!(iter.next().unwrap().is_referenced(), false); // svg
    /// assert_eq!(iter.next().unwrap().is_referenced(), true); // linearGradient
    /// ```
    fn is_referenced(&self) -> bool;

    /// Returns true if the current node is a basic shape element.
    ///
    /// List: `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/shapes.html>
    fn is_basic_shape(&self) -> bool;

    /// Returns true if the current node is a shape element.
    ///
    /// List: `path`, `rect`, `circle`, `ellipse`, `line`, `polyline` and `polygon`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/intro.html#TermShape>
    fn is_shape(&self) -> bool;

    /// Returns true if the current node is a container element.
    ///
    /// List: `a`, `defs`, `glyph`, `g`, `marker`, `mask`, `missing-glyph`, `pattern`, `svg`,
    /// `switch` and `symbol`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/intro.html#TermContainerElement>
    fn is_container(&self) -> bool;

    /// Returns true if the current node is a text content element.
    ///
    /// List: `altGlyph`, `textPath`, `text`, `tref` and `tspan`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/intro.html#TermTextContentElement>
    fn is_text_content(&self) -> bool;

    /// Returns true if the current node is a text content child element.
    ///
    /// List: `altGlyph`, `textPath`, `tref` and `tspan`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/intro.html#TermTextContentChildElement>
    fn is_text_content_child(&self) -> bool;

    /// Returns true if the current node is a graphic element.
    ///
    /// List: `circle`, `ellipse`, `image`, `line`, `path`, `polygon`, `polyline`, `rect`,
    /// `text` and `use`.
    ///
    /// Details: <https://www.w3.org/TR/SVG/intro.html#TermGraphicsElement>
    fn is_graphic(&self) -> bool;

    /// Returns true if the current node is a gradient element.
    ///
    /// List: `linearGradient`, `radialGradient`.
    fn is_gradient(&self) -> bool;

    /// Returns true if the current node is a [paint server].
    ///
    /// List: `linearGradient`, `radialGradient` and `pattern`.
    ///
    /// [paint server]: <https://www.w3.org/TR/SVG11/pservers.html#Introduction>
    fn is_paint_server(&self) -> bool;

    /// Returns true if the current node is a [filter primitive].
    ///
    /// List: `feBlend`, `feColorMatrix`, `feComponentTransfer`, `feComposite`,
    /// `feConvolveMatrix`, `feDiffuseLighting`, `feDisplacementMap`, `feFlood`, `feGaussianBlur`,
    /// `feImage`, `feMerge`, `feMorphology`, `feOffset`, `feSpecularLighting`,
    /// `feTile` and `feTurbulence`.
    ///
    /// [filter primitive]: <https://www.w3.org/TR/SVG11/intro.html#TermFilterPrimitiveElement>
    fn is_filter_primitive(&self) -> bool;
}

macro_rules! is_func {
    ($name:ident, $($pattern:tt)+) => (
        fn $name(&self) -> bool {
            if let Some(id) = self.tag_id() {
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

impl ElementType for Node {
    is_func!(is_referenced,
          ElementId::AltGlyphDef
        | ElementId::ClipPath
        | ElementId::Cursor
        | ElementId::Filter
        | ElementId::LinearGradient
        | ElementId::Marker
        | ElementId::Mask
        | ElementId::Pattern
        | ElementId::RadialGradient
        | ElementId::Symbol);

    is_func!(is_basic_shape,
          ElementId::Rect
        | ElementId::Circle
        | ElementId::Ellipse
        | ElementId::Line
        | ElementId::Polyline
        | ElementId::Polygon);

    is_func!(is_shape,
          ElementId::Circle
        | ElementId::Ellipse
        | ElementId::Line
        | ElementId::Path
        | ElementId::Polygon
        | ElementId::Polyline
        | ElementId::Rect);

    is_func!(is_container,
          ElementId::A
        | ElementId::Defs
        | ElementId::Glyph
        | ElementId::G
        | ElementId::Marker
        | ElementId::Mask
        | ElementId::MissingGlyph
        | ElementId::Pattern
        | ElementId::Svg
        | ElementId::Switch
        | ElementId::Symbol);

    is_func!(is_text_content,
          ElementId::AltGlyph
        | ElementId::TextPath
        | ElementId::Text
        | ElementId::Tref
        | ElementId::Tspan);

    is_func!(is_text_content_child,
          ElementId::AltGlyph
        | ElementId::TextPath
        | ElementId::Tref
        | ElementId::Tspan);

    is_func!(is_graphic,
          ElementId::Circle
        | ElementId::Ellipse
        | ElementId::Image
        | ElementId::Line
        | ElementId::Path
        | ElementId::Polygon
        | ElementId::Polyline
        | ElementId::Rect
        | ElementId::Text
        | ElementId::Use);

    is_func!(is_gradient,
          ElementId::LinearGradient
        | ElementId::RadialGradient);

    is_func!(is_paint_server,
          ElementId::LinearGradient
        | ElementId::RadialGradient
        | ElementId::Pattern);

    is_func!(is_filter_primitive,
          ElementId::FeBlend
        | ElementId::FeColorMatrix
        | ElementId::FeComponentTransfer
        | ElementId::FeComposite
        | ElementId::FeConvolveMatrix
        | ElementId::FeDiffuseLighting
        | ElementId::FeDisplacementMap
        | ElementId::FeFlood
        | ElementId::FeGaussianBlur
        | ElementId::FeImage
        | ElementId::FeMerge
        | ElementId::FeMorphology
        | ElementId::FeOffset
        | ElementId::FeSpecularLighting
        | ElementId::FeTile
        | ElementId::FeTurbulence);
}
