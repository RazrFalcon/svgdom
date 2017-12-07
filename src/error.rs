// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgparser;
use simplecss;

use {
    ErrorPos,
};

// TODO: split to Dom errors and Parser errors

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Xml(svgparser::Error, svgparser::ErrorKind) #[doc = "svgparser errors"];
    }

    errors {
        /// If you want to use referenced element inside link attribute,
        /// such element must have a non-empty ID.
        ElementMustHaveAnId {
            display("the element must have an id")
        }

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
        ElementCrosslink {
            display("element crosslink")
        }

        /// Parsed document must have an `svg` element.
        NoSvgElement {
            display("the document does not have an SVG element")
        }

        /// Parsed document must have at least one node.
        EmptyDocument {
            display("the document does not have any nodes")
        }

        /// *libsvgdom* didn't support most of the CSS2 spec.
        UnsupportedCSS(pos: ErrorPos) {
            display("unsupported CSS at {}", pos)
        }

        /// Error during parsing of the CSS2.
        InvalidCSS(pos: ErrorPos) {
            display("invalid CSS at {}", pos)
        }

        /// ENTITY with XML Element data is not supported.
        UnsupportedEntity(pos: ErrorPos) {
            display("unsupported ENTITY data at {}", pos)
        }

        /// We don't support a \<paint\> type with a fallback value and a valid FuncIRI.
        ///
        /// # Examples
        ///
        /// ```text
        /// <linearGradient id="lg1"/>
        /// <rect fill="url(#lg1) red"/>
        /// ```
        UnsupportedPaintFallback(iri: String) {
            display("valid FuncIRI(#{}) with fallback value is not supported", iri)
        }

        /// We don't support `use` elements with a broken filter attribute.
        BrokenFuncIri(iri: String) {
            display("the 'use' element with a broken filter attribute('#{}') is not supported", iri)
        }

        /// UTF-8 processing error.
        InvalidEncoding {
            display("the input data is not a valid UTF-8 string")
        }

        /// Failed to find an attribute, which must be set, during post-processing.
        MissingAttribute(name: String, value: String) {
            display("attribute '{}' is missing in the '{}' element", name, value)
        }

        /// simplecss errors.
        CssError(e: simplecss::Error) {
            display("{:?}", e)
        }
    }
}

impl From<simplecss::Error> for Error {
    fn from(value: simplecss::Error) -> Error {
        ErrorKind::CssError(value).into()
    }
}
