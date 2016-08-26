use std::fmt;

use svgparser::Error as ParseError;
use svgparser::ErrorPos;

/// List of all errors that can occur during processing of SVG DOM.
#[derive(PartialEq)]
pub enum Error {
    /// If you want to use referenced element inside link attribute,
    /// such element must have an non-empty ID.
    ElementMustHaveAnId,
    /// A linked nodes can't reference each other.
    ///
    /// Example:
    /// ```
    /// <linearGradient id="lg1" xlink:href="#lg2"/>
    /// <linearGradient id="lg2" xlink:href="#lg1"/>
    /// ```
    ElementCrosslink,
    /// Error from *libsvgparser*.
    ParseError(ParseError),
    /// Parsed document must have an `svg` element.
    NoSvgElement,
    /// Parsed document must have at least one node.
    EmptyDocument,
    /// *libsvgdom* didn't support most of the CSS2 spec.
    UnsupportedCSS(ErrorPos),
    /// Error during parsing of the CSS2.
    InvalidCSS(ErrorPos),
    /// ENTITY with XML Element data is not supported.
    UnsupportedEntity(ErrorPos),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ElementMustHaveAnId => write!(f, "Element must have an id"),
            Error::ElementCrosslink => write!(f, "Element crosslink"),
            Error::ParseError(e) => write!(f, "{:?}", e),
            Error::NoSvgElement => write!(f, "Document didn't have a svg element"),
            Error::EmptyDocument => write!(f, "Document didn't have any nodes"),
            Error::UnsupportedCSS(ref pos) => write!(f, "Unsupported CSS at: {:?}", pos),
            Error::InvalidCSS(ref pos) => write!(f, "Invalid CSS at: {:?}", pos),
            Error::UnsupportedEntity(ref pos) => write!(f, "Unsupported ENTITY data at: {:?}", pos),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Error {
        Error::ParseError(value)
    }
}
