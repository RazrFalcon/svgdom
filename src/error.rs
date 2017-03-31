// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::str;

use svgparser::Error as ParseError;
use svgparser::ErrorPos;

use simplecss::Error as CssParseError;

/// List of all errors that can occur during processing of the SVG DOM.
#[derive(PartialEq)]
pub enum Error {
    /// If you want to use referenced element inside link attribute,
    /// such element must have a non-empty ID.
    ElementMustHaveAnId,
    /// A linked nodes can't reference each other.
    ///
    /// Example:
    /// ```
    /// <linearGradient id="lg1" xlink:href="#lg2"/>
    /// <linearGradient id="lg2" xlink:href="#lg1"/>
    /// ```
    /// or
    /// ```
    /// <linearGradient id="lg1" xlink:href="#lg1"/>
    /// ```
    ElementCrosslink,
    /// Error from *libsvgparser*.
    ParseError(ParseError),
    /// Error from *simplecss*.
    CssParseError(CssParseError),
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
    /// We don't support a \<paint\> type with a fallback value and a valid FuncIRI.
    ///
    /// Example:
    /// ```
    /// <linearGradient id="lg1"/>
    /// <rect fill="url(#lg1) red"/>
    /// ```
    UnsupportedPaintFallback(String), // FuncIRI name
    /// We don't support `use` elements with a broken filter attribute.
    BrokenFuncIri(String), // FuncIRI name
    /// UTF-8 processing error.
    InvalidEncoding,
    /// UTF-8 processing error.
    Utf8Error(str::Utf8Error),
    /// Failed to resolve an attribute during post-processing.
    UnresolvedAttribute(String), // attribute name
    /// Failed to find an attribute, which must be set, during post-processing.
    MissingAttribute(String, String), // tag name, attribute name
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ElementMustHaveAnId =>
                write!(f, "Element must have an id"),
            Error::ElementCrosslink =>
                write!(f, "Element crosslink"),
            Error::ParseError(e) =>
                write!(f, "{}", e),
            Error::CssParseError(e) =>
                write!(f, "{:?}", e),
            Error::NoSvgElement =>
                write!(f, "Document didn't have an SVG element"),
            Error::EmptyDocument =>
                write!(f, "Document didn't have any nodes"),
            Error::UnsupportedCSS(ref pos) =>
                write!(f, "Unsupported CSS at {}", pos),
            Error::InvalidCSS(ref pos) =>
                write!(f, "Invalid CSS at {}", pos),
            Error::UnsupportedEntity(ref pos) =>
                write!(f, "Unsupported ENTITY data at {}", pos),
            Error::UnsupportedPaintFallback(ref iri) =>
                write!(f, "Valid FuncIRI(#{}) with fallback value is not supported", iri),
            Error::BrokenFuncIri(ref iri) =>
                write!(f, "The 'use' element with a broken filter attribute('#{}') \
                           is not supported", iri),
            Error::InvalidEncoding =>
                write!(f, "The input data is not a valid UTF-8 string"),
            Error::Utf8Error(e) =>
                write!(f, "{}", e),
            Error::UnresolvedAttribute(ref name) =>
                write!(f, "Failed to resolved attribute '{}'", name),
            Error::MissingAttribute(ref tag_name, ref attr_name) =>
                write!(f, "The attribute '{}' is missing in the '{}' element", attr_name, tag_name),
        }
    }
}

// TODO: is needed?
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

impl From<CssParseError> for Error {
    fn from(value: CssParseError) -> Error {
        Error::CssParseError(value)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(value: str::Utf8Error) -> Error {
        Error::Utf8Error(value)
    }
}
