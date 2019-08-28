## svgdom
[![Build Status](https://travis-ci.org/RazrFalcon/svgdom.svg?branch=master)](https://travis-ci.org/RazrFalcon/svgdom)
[![Crates.io](https://img.shields.io/crates/v/svgdom.svg)](https://crates.io/crates/svgdom)
[![Documentation](https://docs.rs/svgdom/badge.svg)](https://docs.rs/svgdom)


*svgdom* is an [SVG Full 1.1](https://www.w3.org/TR/SVG/) processing library,
which allows you to parse, manipulate, generate and write an SVG content.

### Deprecation

This library was an attempt to create a generic SVG DOM which can be used by various applications.
But it the end it turned out that it's easier and faster to use
[roxmltree](https://github.com/RazrFalcon/roxmltree) + [svgtypes](https://github.com/RazrFalcon/svgtypes)
to extract only the data you need.

There are two main problems with `svgdom`:

1. You can't make a nice API with a Vec-based tree and you can't have a safe API
   with an Rc-tree.

   The current implementation uses so-called Rc-tree, which provides a nice API,
   but all the checks are done in the runtime, so you can get a panic quite easily.
   It's also hard/verbose to make immutable nodes. You essentially need two types of nodes:
   one for immutable and one for mutable "references".
   A Vec-based tree would not have such problems, but you can't implement the simplest
   operations with it, like copying an attribute from one node to another
   since you have to have a mutable and an immutable references for this.
   And Rust forbids this. So you need some sort of generational indexes and so on.
   This solution is complicated in its own way.
   Performance is also in question, since inserting/removing an object in the middle of a Vec is expensive.
2. The SVG parsing itself is pretty complex too. There are a lot of ways you can implement it.

   `svgdom` creates a custom Rc-tree where all the attributes are stored as owned data.
   This requires a lot of allocations (usually unnecessary).
   The parsing/preprocessing algorithm itself can be found in [docs/preprocessor.md](docs/preprocessor.md)
   The problem with it is that you can't tweak it. And in many cases, it produces results
   that you do not need or do not expect.
   `svgdom` was originally used by [svgcleaner](https://github.com/RazrFalcon/svgcleaner)
   and [resvg](https://github.com/RazrFalcon/resvg) and both of these projects are no longer using it.

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
- SVG 1.1 Full only (no 2.0 Draft, Basic, Tiny subsets).

### Differences between svgdom and SVG spec

- Library follows SVG spec in the data parsing, writing, but not in the tree structure.
- Everything is a `Node`. There are no separated `ElementNode`, `TextNode`, etc.
  You still have all the data, but not in the specific *struct's*.
  You can check the node type via `Node::node_type()`.


### Dependency

[Rust](https://www.rust-lang.org/) >= 1.32

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
