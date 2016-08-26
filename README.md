## libsvgdom

**libsvgdom** is a [SVG Full 1.1](https://www.w3.org/TR/SVG/) processing library,
which allows you to parse, manipulate and generate SVG content.

### Purpose

**libsvgdom** is designed to simplify generic SVG processing and manipulations.
Unfortunately, SVG is very complex format (PDF spec is 826 pages long),
with lots of features and implementing all of them will lead to a enormous library.

That's why libsvgdom supports only static subset of SVG. No scripts, external resources and complex
CSS styling.
Parser will convert as much as possible data to simple doc->elements->attributes structure.

For example, the `fill` parameter of element can be set: as an element's attribute,
as part of the `style` attribute, inside CDATA of the *style* element as CSS2, inside the `ENTITY`,
using the JS code and probably with a lots of other methods.

Not to mention, that the `fill` attribute supports a 4 different types of data.

With libsvgdom you can just use `node.has_attribute(AttributeId::Fill)` and don't worry were this
attribute was defined in original file.

Same goes to the transforms, paths and other SVG types.

Main downside of this approach is that you can't save original formatting and some data.

### Benefits
 - Element link(IRI, FuncIRI) not just text, but actual link to another node.
 - At any time you can check which element linked to selected element. See `Node` doc for details.
 - Lots of options which are controls data loading and saving.
 - See [libsvgparser](https://github.com/RazrFalcon/libsvgparser)'s README for parsing benefits.

### Limitations
 - This library doesn't implement full SVG spec and newer will be.
   Treat it like extended XML processing library, with SVG specific features.
 - Because we converts attributes, CDATA, DOCTYPE data to internal representation - we
   cannot save original content, formatting, etc.
 - Encoding should be UTF-8.
 - Only most popular attributes are parsed, other stored as strings.
 - Compressed SVG (.svgz). You should decompress it by yourself.
 - Not supported (mostly rare cases, but still valid by the SVG spec):
   - Complex CSS. Only simple *class* and *group* selectors are supported.
   - Whitespacing using a numerical Unicode references, aka `&#x0020;`.
   - Custom namespaces, like
      ```
      <g xmlns:s="http://www.w3.org/2000/svg">
        <s:circle/>
      </g>
      ```
      It will be threated as non-SVG element.
   - Links to ENTITY not from attributes will lead to `Error::UnsupportedEntity`. Example:
      ```
      <!DOCTYPE svg [
         <!ENTITY Rect1 "<rect x='.5' y='.5' width='20' height='20'/>">
        ]>
      <svg>&Rect1;</svg>
      ```
 - See [libsvgparser](https://github.com/RazrFalcon/libsvgparser)'s README for parsing limitations.

### Non-goal
 - Implementation of full SVG spec.
 - Animation support.
 - Scripting support (meaning via `script` element).

### Differences between *libsvgdom* and SVG spec
 - Library follows SVG spec in the data parsing, writing, but not in tree structure.
 - Everything is a `Node`. There are no separated `ElementNode`, `TextNode`, etc.
   You still have all the data, but not in the specific *struct's*.
   You can check a node type via `Node::node_type()`.
 - *FuncIRI* and *IRI* are the same types.

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
svgdom = "0.0.1"
```

See [documentation](https://razrfalcon.github.io/libsvgdom/svgdom/index.html)
and [examples](examples/) for details.

### Performance

It will be no comparisons with other XML parsers, since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such\*.

Note that most of the time is spend in string to number and number to string conversions.

It's still not as as fast as I want, but here is some examples using *resave* example:

[Some huge image](https://openclipart.org/detail/259586/cyberscooty-floral-border-extended-22)\*\*
(17.3MiB): ~700ms/~5500M instructions.

[Big image](https://en.wikipedia.org/wiki/File:Jupiter_diagram.svg)
(1.7MiB): ~70ms/~500M instructions.

[Average image](https://commons.wikimedia.org/wiki/File:Electromagnetic_Radiation_Spectrum_Infographic.svg)
(324.4KiB): ~20ms/~89M instructions.

Small image, like [SVG Logo](https://commons.wikimedia.org/wiki/File:SVG_logo.svg)
(8.8KiB): ~0.4ms/~2M instructions.

\* At least I don't know.

\*\* It's not a direct download links.

Tested on i5-3570k@4.2GHz.

### Roadmap

V0.1.0
 - [ ] Increase performance:
   - [ ] `f64` to string conversion takes about 75% off all DOM to String conversion time.
         Specifically: `core::fmt::float_to_decimal_common`.
 - Improve grammar of the documentation.
   English is not my native language, so there is probably a lot of errors.

V0.2.0
 - Parsing and writing as a feature.
 - Add support for custom writer.
 - Memory pool for nodes.
 - Implement DOM with less `Rc`.

V0.N.0
 - Complete CSS support using external CSS parser.

\* this roadmap is not final and will be complemented.

### Contributing

Contributions are welcome, but current API is so much unstable, that it's better to wait until
v0.1.0.

### License

libsvgdom is licensed under the [MPLv2.0](https://www.mozilla.org/en-US/MPL/).
