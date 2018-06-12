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

### Example

Original image:
```svg
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE svg [
    <!ENTITY ns_xlink "http://www.w3.org/1999/xlink">
    <!ENTITY color "red">
]>
<!-- Comment -->
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="&ns_xlink;">
    <defs>
        <radialGradient id="rg1">
            <stop offset="0" stop-color="yellow"/>
            <stop offset="1" stop-color="green"/>
        </radialGradient>
    </defs>
    <style type='text/css'>
        <![CDATA[
            .fill1 { fill:#00913f }
        ]]>
    </style>
    <rect fill="url(#rg1)" stroke="url(#lg1)" x="5" y="5" width="50" height="50"/>
    <rect class="fill1" y="2.5" width="25" height="25" transform="scale(2) translate(30)"/>
    <rect style="fill:blue" stroke="&color;" x="115" y="5" width="50" height="50"/>
    <text x="60" y="1.5em">Text</text>
    <path stroke="red" d="M 165 60l-160 0 #L 50 100"/>
    <myelement myattribute="value"/>
</svg>
```

How it will be represented and saved using svgdom:
```svg
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!-- Comment -->
<svg height="70" width="175" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <radialGradient id="rg1">
            <stop offset="0" stop-color="#ffff00"/>
            <stop offset="1" stop-color="#008000"/>
        </radialGradient>
    </defs>
    <rect fill="url(#rg1)" height="50" stroke="url(#lg1)" width="50" x="5" y="5"/>
    <rect fill="#00913f" height="25" transform="matrix(2 0 0 2 60 0)" width="25" y="2.5"/>
    <rect fill="#0000ff" height="50" stroke="#ff0000" width="50" x="115" y="5"/>
    <text x="60" y="1.5em">Text</text>
    <path d="M 165 60 l -160 0" stroke="#ff0000"/>
    <myelement myattribute="value"/>
</svg>
```

And even though the file is a bit different now - it will be rendered exactly the same.

![Alt text](https://cdn.rawgit.com/RazrFalcon/svgdom/master/examples/images/image_before.svg)

![Alt text](https://cdn.rawgit.com/RazrFalcon/svgdom/master/examples/images/image_after.svg)

### Benefits
 - The element link(IRI, FuncIRI) is not just a text, but an actual link to another node.
 - At any time you can check which elements linked to the specific element.
   See `Node`'s doc for details.
 - Support for many SVG specific data types like paths, transforms, IRI's, styles, etc.
   Thanks to [svgtypes](https://github.com/RazrFalcon/svgtypes). 
 - A complete support of text nodes: XML escaping, `xml:space`.
 - Fine-grained control over the SVG output.

### Limitations
 - Attribute values, CDATA with CSS, DOCTYPE, text data and whitespaces will not be preserved.
 - UTF-8 only.
 - Only most popular attributes are parsed, other stored as strings.
 - No compressed SVG (.svgz). You should decompress it by yourself.
 - CSS support is minimal.
 - XML namespaces support is minimal.
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
svgdom = "0.13"
```

See [documentation](https://docs.rs/svgdom/) and [examples](examples/) for details.

### Performance

There will be no comparisons with other XML parsers since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such.

Note that most of the time is spent during string to number and number to string conversion.

```
test parse_large  ... bench:  16,089,315 ns/iter (+/- 370,141)
test parse_medium ... bench:   2,869,603 ns/iter (+/- 3,144)
test parse_small  ... bench:      57,240 ns/iter (+/- 80)
test write_large  ... bench:  13,055,774 ns/iter (+/- 55,332)
test write_medium ... bench:   1,309,112 ns/iter (+/- 1,722)
test write_small  ... bench:      26,017 ns/iter (+/- 87)
```

Tested on i5-3570k 3.4GHz.

### License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
