// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt;

use simplecss;

use svgtypes;
use svgtypes::xmlparser::{
    self,
    ErrorPos,
};

// TODO: split to Dom errors and Parser errors

/// SVG DOM errors.
#[derive(Debug)]
pub enum Error {
    /// If you want to use referenced element inside link attribute,
    /// such element must have a non-empty ID.
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
    ElementCrosslink,

    /// Parsed document must have an `svg` element.
    NoSvgElement,

    /// Parsed document must have at least one node.
    EmptyDocument,

    /// *svgdom* didn't support most of the CSS2 spec.
    UnsupportedCSS(ErrorPos),

    /// Error during parsing of the CSS2.
    InvalidCSS(ErrorPos),

    /// Unexpected close tag.
    ///
    /// # Examples
    ///
    /// ```text
    /// <svg>
    ///     </rect>
    /// </svg>
    /// ```
    UnexpectedCloseTag(String, String),

    /// Error during attribute value parsing.
    SvgTypesError(svgtypes::Error),

    /// An XML stream error.
    XmlError(xmlparser::Error),

    /// simplecss errors.
    CssError(simplecss::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ElementMustHaveAnId => {
                write!(f, "the element must have an id")
            }
            Error::ElementCrosslink => {
                write!(f, "element crosslink")
            }
            Error::NoSvgElement => {
                write!(f, "the document does not have an SVG element")
            }
            Error::EmptyDocument => {
                write!(f, "the document does not have any nodes")
            }
            Error::UnsupportedCSS(pos) => {
                write!(f, "unsupported CSS at {}", pos)
            }
            Error::InvalidCSS(pos) => {
                write!(f, "invalid CSS at {}", pos)
            }
            Error::UnexpectedCloseTag(ref first, ref second) => {
                write!(f, "opening and ending tag mismatch '{}' and '{}'", first, second)
            }
            Error::SvgTypesError(ref e) => {
                write!(f, "{}", e)
            }
            Error::XmlError(ref e) => {
                write!(f, "{}", e)
            }
            Error::CssError(ref e) => {
                write!(f, "{:?}", e) // TODO: impl Display for simplecss's Error
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "an SVG parsing error"
    }
}

impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Error {
        Error::XmlError(value)
    }
}

impl From<svgtypes::Error> for Error {
    fn from(value: svgtypes::Error) -> Error {
        Error::SvgTypesError(value)
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
pub(crate) type Result<T> = ::std::result::Result<T, Error>;
