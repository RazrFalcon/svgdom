## svgdom
[![Build Status](https://travis-ci.org/RazrFalcon/svgdom.svg?branch=master)](https://travis-ci.org/RazrFalcon/svgdom)
[![Crates.io](https://img.shields.io/crates/v/svgdom.svg)](https://crates.io/crates/svgdom)
[![Documentation](https://docs.rs/svgdom/badge.svg)](https://docs.rs/svgdom)

*svgdom* is an [SVG Full 1.1](https://www.w3.org/TR/SVG/) processing library,
which allows you to parse, manipulate, generate and write an SVG content.

**Note:** the library itself is pretty stable, but API is constantly changing.

### Purpose

*svgdom* is designed to simplify generic SVG processing and manipulations.
Unfortunately, an SVG is very complex format (PDF spec is 826 pages long),
with lots of features and implementing all of them will lead to an enormous library.

That's why *svgdom* supports only a static subset of an SVG. No scripts, external resources
and complex CSS styling.
Parser will convert as much as possible data to a simple doc->elements->attributes structure.

For example, the `fill` parameter of an element can be set: as an element's attribute,
as part of a `style` attribute, inside a `style` element as CSS2, inside an `ENTITY`,
using a JS code and probably with lots of other methods.

Not to mention, that the `fill` attribute supports 4 different types of data.

With `svgdom` you can just use `node.has_attribute(AttributeId::Fill)` and don't worry where this
attribute was defined in the original file.

Same goes for transforms, paths and other SVG types.

The main downside of this approach is that you can't save an original formatting and some data.

See the [preprocessor](https://github.com/RazrFalcon/svgdom/blob/master/docs/preprocessor.md)
doc for details.

### Benefits

- The element link(IRI, FuncIRI) is not just a text, but an actual link to another node.
- At any time you can check which elements linked to the specific element.
  See `Node`'s doc for details.
- Support for many SVG specific data types like paths, transforms, IRI's, styles, etc.
  Thanks to [svgtypes](https://github.com/RazrFalcon/svgtypes).
- A complete support of text nodes: XML escaping, `xml:space`.
- Fine-grained control over the SVG output.

### Limitations

- Only SVG elements and attributes will be parsed.
- Attribute values, CDATA with CSS, DOCTYPE, text data and whitespaces will not be preserved.
- UTF-8 only.
- Only most popular attributes are parsed, other stored as strings.
- No compressed SVG (.svgz). You should decompress it by yourself.
- CSS support is minimal.
- SVG 1.1 Full only (no 2.0 Draft, Basic, Type subsets).

### Differences between svgdom and SVG spec

- Library follows SVG spec in the data parsing, writing, but not in the tree structure.
- Everything is a `Node`. There are no separated `ElementNode`, `TextNode`, etc.
  You still have all the data, but not in the specific *struct's*.
  You can check the node type via `Node::node_type()`.


### Usage

Dependency: [Rust](https://www.rust-lang.org/) >= 1.18

Add this to your `Cargo.toml`:

```toml
[dependencies]
svgdom = "0.14"
```

### License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
