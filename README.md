## libsvgdom [![Build Status](https://travis-ci.org/RazrFalcon/libsvgdom.svg?branch=master)](https://travis-ci.org/RazrFalcon/libsvgdom)

*libsvgdom* is an [SVG Full 1.1](https://www.w3.org/TR/SVG/) processing library,
which allows you to parse, manipulate and generate SVG content.

**Note:** the library itself is pretty stable, but API is constantly changing.

## Table of Contents

 - [Purpose](#purpose)
    - [Example](#example)
 - [Documentation](#documentation)
 - [Benefits](#benefits)
 - [Limitations](#limitations)
 - [Non-goal](#non-goal)
 - [Differences between libsvgdom and SVG spec](#differences-between-libsvgdom-and-svg-spec)
 - [Usage](#usage)
 - [Performance](#performance)
 - [Contributing](#contributing)
 - [License](#license)

### Purpose

*libsvgdom* is designed to simplify generic SVG processing and manipulations.
Unfortunately, an SVG is very complex format (PDF spec is 826 pages long),
with lots of features and implementing all of them will lead to an enormous library.

That's why *libsvgdom* supports only static subset of an SVG. No scripts, external resources
and complex CSS styling.
Parser will convert as much as possible data to a simple doc->elements->attributes structure.

For example, the `fill` parameter of an element can be set: as an element's attribute,
as part of a `style` attribute, inside a `style` element as CSS2, inside an `ENTITY`,
using a JS code and probably with lots of other methods.

Not to mention, that the `fill` attribute supports 4 different types of data.

With `libsvgdom` you can just use `node.has_attribute(AttributeId::Fill)` and don't worry where this
attribute was defined in the original file.

Same goes for transforms, paths and other SVG types.

The main downside of this approach is that you can't save original formatting and some data.

#### Example

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
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <defs>
        <radialGradient id="rg1">
            <stop offset="0" stop-color="#ffff00"/>
            <stop offset="1" stop-color="#008000"/>
        </radialGradient>
    </defs>
    <rect fill="url(#rg1)" height="50" width="50" x="5" y="5"/>
    <rect fill="#00913f" height="25" transform="matrix(2 0 0 2 60 0)" width="25" y="2.5"/>
    <rect fill="#0000ff" height="50" stroke="#ff0000" width="50" x="115" y="5"/>
    <text x="60" y="1.5em">
        Text
    </text>
    <path d="M 165 60 l -160 0" stroke="#ff0000"/>
    <myelement myattribute="value"/>
</svg>
```

And even though the file is a bit different now - it will be rendered exactly the same.

![Alt text](https://cdn.rawgit.com/RazrFalcon/libsvgdom/master/examples/images/image_before.svg)

![Alt text](https://cdn.rawgit.com/RazrFalcon/libsvgdom/master/examples/images/image_after.svg)

### [Documentation](https://docs.rs/svgdom/)

### Benefits
 - The element link(IRI, FuncIRI) is not just text, but actual link to another node.
 - At any time you can check which elements linked to the selected element. See `Node` doc for details.
 - A complete support of text nodes: XML escaping, `xml:space`.
 - Many options that control data loading and saving.
 - See [libsvgparser](https://github.com/RazrFalcon/libsvgparser)'s README for parsing benefits.

### Limitations
 - Because we convert attributes, CDATA, DOCTYPE data to internal representation - we
   cannot save original content, formatting, etc.
 - Encoding should be UTF-8.
 - Only most popular attributes are parsed, other stored as strings.
 - Compressed SVG (.svgz). You should decompress it by yourself.
 - Not supported (mostly rare cases, but still valid by the SVG spec):
   - Complex CSS. Only simple selectors are supported.
   - Custom namespaces, like:

      ```
      <g xmlns:s="http://www.w3.org/2000/svg">
        <s:circle/>
      </g>
      ```
      It will be treated as a non-SVG element.
   - Links to ENTITY not from attributes will lead to `Error::UnsupportedEntity`. Example:

     ```
     <!DOCTYPE svg [
        <!ENTITY Rect1 "<rect x='.5' y='.5' width='20' height='20'/>">
       ]>
     <svg>&Rect1;</svg>
     ```
 - See [libsvgparser](https://github.com/RazrFalcon/libsvgparser)'s README for parsing limitations.

### Non-goal
 - Implementation of the full SVG spec.
 - Animation support.
 - Scripting support (via `script` element).

### Differences between libsvgdom and SVG spec
 - Library follows SVG spec in the data parsing, writing, but not in the tree structure.
 - Everything is a `Node`. There are no separated `ElementNode`, `TextNode`, etc.
   You still have all the data, but not in the specific *struct's*.
   You can check a node type via `Node::node_type()`.

### Usage

Dependency: [Rust](https://www.rust-lang.org/) >= 1.18

Add this to your `Cargo.toml`:

```toml
[dependencies]
svgdom = "0.11"
```

See [documentation](https://docs.rs/svgdom/) and [examples](examples/) for details.

### Performance

There will be no comparisons with other XML parsers since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such.

Note that most of the time is spent during string to number and number to string conversion.

```
test parse_large  ... bench:  13,410,092 ns/iter (+/- 181,693)
test parse_medium ... bench:   2,868,389 ns/iter (+/- 16,071)
test parse_small  ... bench:      61,976 ns/iter (+/- 235)
test write_large  ... bench:  12,606,581 ns/iter (+/- 71,450)
test write_medium ... bench:   1,288,102 ns/iter (+/- 25,051)
test write_small  ... bench:      24,420 ns/iter (+/- 87)
```

Tested on i5-3570k 3.4GHz.

### License

*libsvgdom* is licensed under the [MPLv2.0](https://www.mozilla.org/en-US/MPL/).
