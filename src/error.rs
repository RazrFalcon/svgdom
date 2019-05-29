use std::error;
use std::fmt;

use roxmltree::{self, TextPos};

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

    /// A DOM API error.
    DomError(Error),

    /// An invalid attribute value.
    InvalidAttributeValue(TextPos),

    /// A `roxmltree` error.
    RoXmlError(roxmltree::Error),
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
            ParserError::InvalidAttributeValue(pos) => {
                write!(f, "invalid attribute value at {}", pos)
            }
            ParserError::DomError(ref e) => {
                write!(f, "{}", e)
            }
            ParserError::RoXmlError(ref e) => {
                write!(f, "{}", e)
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

impl From<roxmltree::Error> for ParserError {
    fn from(value: roxmltree::Error) -> Self {
        ParserError::RoXmlError(value)
    }
}
