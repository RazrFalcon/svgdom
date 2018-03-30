// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use simplecss;

use svgparser;
use svgparser::xmlparser::{
    self,
    ErrorPos,
};

// TODO: split to Dom errors and Parser errors

/// SVG DOM errors.
#[derive(Fail, Debug)]
pub enum Error {
    /// If you want to use referenced element inside link attribute,
    /// such element must have a non-empty ID.
    #[fail(display = "the element must have an id")]
    ElementMustHaveAnId,

    /// Linked nodes can't reference each other or itself.
    ///
    /// # Examples
    ///
    /// ```text
    /// <linearGradient id="lg1" xlink:href="#lg2"/>
    /// <linearGradient id="lg2" xlink:href="#lg1"/>
    /// ```
    ///
    /// or
    ///
    /// ```text
    /// <linearGradient id="lg1" xlink:href="#lg1"/>
    /// ```
    #[fail(display = "element crosslink")]
    ElementCrosslink,

    /// Parsed document must have an `svg` element.
    #[fail(display = "the document does not have an SVG element")]
    NoSvgElement,

    /// Parsed document must have at least one node.
    #[fail(display = "the document does not have any nodes")]
    EmptyDocument,

    /// *libsvgdom* didn't support most of the CSS2 spec.
    #[fail(display = "unsupported CSS at {}", _0)]
    UnsupportedCSS(ErrorPos),

    /// Error during parsing of the CSS2.
    #[fail(display = "invalid CSS at {}", _0)]
    InvalidCSS(ErrorPos),

    /// ENTITY with XML Element data is not supported.
    #[fail(display = "unsupported ENTITY data at {}", _0)]
    UnsupportedEntity(ErrorPos),

    /// We don't support a \<paint\> type with a fallback value and a valid FuncIRI.
    ///
    /// # Examples
    ///
    /// ```text
    /// <linearGradient id="lg1"/>
    /// <rect fill="url(#lg1) red"/>
    /// ```
    #[fail(display = "valid FuncIRI(#{}) with fallback value is not supported", _0)]
    UnsupportedPaintFallback(String),

    // TODO: only `use`?
    /// We don't support `use` elements with a broken filter attribute.
    #[fail(display = "the 'use' element with a broken filter attribute('#{}') is not supported", _0)]
    BrokenFuncIri(String),

    /// Failed to find an attribute, which must be set, during post-processing.
    #[fail(display = "attribute '{}' is missing in the '{}' element", _0, _1)]
    MissingAttribute(String, String),

    /// Error during attribute value parsing.
    #[fail(display = "invalid attribute value cause {}", _0)]
    InvalidAttributeValue(svgparser::StreamError),

    /// An XML stream error.
    #[fail(display = "{}", _0)]
    XmlError(xmlparser::Error),

    /// simplecss errors.
    #[fail(display = "{:?}", _0)]
    CssError(simplecss::Error),
}

impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Error {
        Error::XmlError(value)
    }
}

impl From<svgparser::StreamError> for Error {
    fn from(value: svgparser::StreamError) -> Error {
        Error::InvalidAttributeValue(value)
    }
}

impl From<simplecss::Error> for Error {
    fn from(value: simplecss::Error) -> Error {
        Error::CssError(value)
    }
}

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;
