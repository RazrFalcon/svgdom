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

use roxmltree;

use svgtypes;
use svgtypes::xmlparser::{
    self,
    TextPos,
};

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
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "an SVG error"
    }
}


/// SVG parsing errors.
#[derive(Debug)]
pub enum ParserError {
    /// Parsed document must have an `svg` element.
    NoSvgElement,

    /// *svgdom* didn't support most of the CSS2 spec.
    UnsupportedCSS(TextPos),

    /// Error during parsing of the CSS2.
    InvalidCSS(TextPos),

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

    /// A DOM API error.
    DomError(Error),

    /// Error during attribute value parsing.
    SvgTypesError(svgtypes::Error),

    /// An XML stream error.
    XmlError(xmlparser::Error),

    /// A `roxmltree` error.
    RoXmlError(roxmltree::Error),

    /// simplecss errors.
    CssError(simplecss::Error),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::NoSvgElement => {
                write!(f, "the document does not have an SVG element")
            }
            ParserError::UnsupportedCSS(pos) => {
                write!(f, "unsupported CSS at {}", pos)
            }
            ParserError::InvalidCSS(pos) => {
                write!(f, "invalid CSS at {}", pos)
            }
            ParserError::UnexpectedCloseTag(ref first, ref second) => {
                write!(f, "opening and ending tag mismatch '{}' and '{}'", first, second)
            }
            ParserError::DomError(ref e) => {
                write!(f, "{}", e)
            }
            ParserError::SvgTypesError(ref e) => {
                write!(f, "{}", e)
            }
            ParserError::XmlError(ref e) => {
                write!(f, "{}", e)
            }
            ParserError::RoXmlError(ref e) => {
                write!(f, "{}", e)
            }
            ParserError::CssError(ref e) => {
                write!(f, "{:?}", e) // TODO: impl Display for simplecss's Error
            }
        }
    }
}

impl error::Error for ParserError {
    fn description(&self) -> &str {
        "an SVG parsing error"
    }
}

impl From<Error> for ParserError {
    fn from(value: Error) -> Self {
        ParserError::DomError(value)
    }
}

impl From<xmlparser::Error> for ParserError {
    fn from(value: xmlparser::Error) -> Self {
        ParserError::XmlError(value)
    }
}

impl From<roxmltree::Error> for ParserError {
    fn from(value: roxmltree::Error) -> Self {
        ParserError::RoXmlError(value)
    }
}

impl From<svgtypes::Error> for ParserError {
    fn from(value: svgtypes::Error) -> Self {
        ParserError::SvgTypesError(value)
    }
}

impl From<simplecss::Error> for ParserError {
    fn from(value: simplecss::Error) -> Self {
        ParserError::CssError(value)
    }
}
