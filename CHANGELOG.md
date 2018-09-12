# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.14.0] - 2018-09-12
### Added
- Implemented `Display` for `Attributes` and `Node`.
- Implemented `Debug` for `ParseOptions` and `WriteOptions`.

### Changed
- From now, only SVG elements and attributes will be parsed.
- Split `Error` into `Error` and `ParserError`.
- New `Debug` implementation for `Attributes` and `Node`.
- `Attributes::new`, `Attributes::insert` and `Attributes::remove` are private now.
- Rename `Attribute::default` into `new_default`.

### Removed
- Namespace prefixes.
- `failure` dependency.
- Unused error types.
- `NodeType::Declaration`, `Node::is_declaration`.
- `NodeType::Cdata`, `Node::is_cdata`.
- `parse_comments`, `parse_declarations`, `parse_unknown_elements`, `parse_unknown_attributes`,
  `parse_px_unit`, `skip_elements_crosslink`, `skip_paint_fallback` from `ParseOptions`.
- `assert_eq_text` macro.
- `Attributes::insert_from`. Use `Node::set_attribute` instead,
- `Attributes::remove_impl`.
- `Attributes::retain`.
- `Node::has_attributes`.
- `Attribute::check_is_default`.

## [0.13.0] - 2018-05-23
### Added
- Check for a proper element opening and closing tags.
- `is_root`, `is_element`, `is_declaration`, `is_comment`, `is_cdata` and `is_text`
  methods to the `Node` struct.
- `is_none`, `is_inherit`, `is_current_color`, `is_paint` and `is_aspect_ratio`
  methods to the `Attribute` struct.
- Implement all `is_*` methods form the `Attribute` struct to `AttributeValue`.
- `AttributeValue::Paint`.
- `ElementType::is_paint_server`.
- `is_link_container` to `Attribute` and `AttributeValue`.
- `Node::is_detached`.
- Elements from ENTITY resolving.

### Changed
- FuncIRI for `fill` and `stroke` attributes will be parsed as
  `AttributeValue::Paint` and not as `AttributeValue::FuncLink` now.
- `Document::create_node` accepts `Into<String>` and not `&str` now.
- `Declaration` node type accepts attributes now. So it will be parsed as node
  with attributes and not as node with text.
- Not well-defined `id` attributes are allowed now.
- Parse `rotate` attribute as `NumberList`.
- New text preprocessing algorithm.

### Removed
- `Attribute::visible` field.
- `Node::has_visible_attribute`.
- `WriteOptions::write_hidden_attributes`.
- `Attributes::iter_svg`. Use `iter().svg()` instead.
- `Attributes::iter_svg_mut`. Use `iter_mut().svg()` instead.

### Fixed
- Mixed `xml:space` processing.
- Empty `tspan` saving.

## [0.12.0] - 2018-04-24
### Added
- `ParseOptions::skip_elements_crosslink`.
- Implemented `WriteBuffer` and `Display` for `QName`.

### Changed
- All SVG types implementation move to the `svgtypes` crate.
- `Node::set_attribute_if_none` accepts `Into<Attribute>` now.

### Removed
- `ValueId` type. Now only `none`, `inherit` and `currentColor`
   will be stored not as strings.

## [0.11.1] - 2018-04-12
- Moved to `rctree` as a tree backend.

## [0.11.0] - 2018-04-10
### Added
- Implemented `Deref` and `DerefMut` for `Path`.
- `AttributeValue::Points`.
- `AttributeValue::ViewBox`.
- `AttributeValue::AspectRatio`.

### Changed
- Moved to `failure`.
- Moved to `rcc-tree` from own tree implementation.
- Relicense from MPL-2.0 to MIT/Apache-2.0.
- Minimal Rust version is 1.18.
- `Path`'s fields are private now. Use `Deref` instead.
- `Attribute::name` is `QName` and not `Name` now.
  So it has namespace prefix in it. Example:
  ```rust
  // before
  node.set_attribute(AttributeId::XmlSpace, "preserve");
  // from now
  node.set_attribute(("xml", AttributeId::Space)), "preserve");
  ```
- `Name::into_ref` to `Name::as_ref`.
- `Node::tag_name` returns `Ref<TagName>`
  and not `Option<Ref<TagName>>` now.
- Rename `Node::parents` to `ancestors`.
- `Node::ancestors`(parents) starts with a current node now and returns the root node too.

### Removed
- `AttributeValue::name`.
- `Node::document`.
- `Node::remove`. Use `Document::remove_node`.
- `Node::remove_attributes`.
- `Node::remove_attributes`.
- `Node::make_copy`. Use `Document::copy_node`.
- `Node::make_copy_deep`. Use `Document::copy_node_deep`.
- `Document::first_child`.
- `Document::append`.
- `Document::descendants`.
- `Document::children`.
- `LinkedNodes` iterator.

## [0.10.5] - 2018-04-10
### Changed
- A default `Transform` will be printed as `matrix(1 0 0 1 0)` and not as an empty string.

### Fixed
- Text with `xml:space` preprocessing.

## [0.10.4] - 2018-02-03
### Fixed
- Invalid files in the crate package.

## [0.10.3] - 2018-01-29
### Fixed
- Memory leak.
- Stack overflow when `Document` has a lot of nodes (>100k).

## [0.10.2] - 2018-01-23
### Fixed
- `WriteOptions::remove_duplicated_path_commands` ignores `MoveTo` now.

## [0.10.1] - 2018-01-17
### Fixed
- `marker` property resolving from CSS.

## [0.10.0] - 2018-01-17
**Note:** this update contain breaking changes.

### Added
- `WriteOptions::list_separator`.
- `WriteOptions::attributes_order`.
- Implemented `WriteBuffer` and `ToStringWithOptions` for `NumberList` and `LengthList`.
- Quotes escape in attribute values.

### Changed
- The `types` module is private now and all types are available in the global namespace.
- `WriteOptionsPaths` merged to `WriteOptions`.
- The value of the `unicode` attribute is always escaped now.
- The minimal Rust version is 1.16 now. Because of `log`.

## [0.9.1] - 2017-12-15
### Fixed
- Text saving.

## [0.9.0] - 2017-12-15
**Note:** this update contain breaking changes.

### Added
- `Node::set_attribute_if_none`.
- Better text parsing.
- Implemented `AttributeType` for `AttributeId`.
- `Option::skip_invalid_attributes`.
- `Option::skip_invalid_css`.
- `Option::skip_paint_fallback`.

### Changed
- `Descendants::svg`, `Children::svg` and `Parents::svg`
  returns `(ElementId, Node)` instead of `Node` now.
- Errors implemented via `error-chain` now.
- Quotes inside text nodes no longer escaped.
- All warnings will be printed with `warn!` macro from the `log` crate now.
- Rename `FromFrame` to `ParseFromSpan`.

### Removed
- `postproc` module.

## [0.8.1] - 2017-10-02
### Fixed
- Memory leak.

## [0.8.0] - 2017-09-30
### Changed
- Rename `FromStream` into `FromFrame`.
- `FromFrame` no longer implements the `from_str` method
  and inherits from `FromStr` instead.
- Rename `WriteToString` into `ToStringWithOptions`.

### Removed
- `parsing` build feature.

## [0.7.0] - 2017-09-26
### Added
- `FuzzyEq::is_fuzzy_zero`.
- `Node::parents_with_self`.
- `WriteOptions::attributes_indent`.
- `Length::new_number`.
- Text escaping before saving to file.
- **Breaking change.** Enforced mutability. Many `Document`'s
  and `Node`'s methods require `&mut self` now.
  This will prevent many runtime errors caused by borrowing `Rc` as mutable more than once.
- `Debug` for `Attributes`.

### Changed
- `writer` module is private now.
- **Breaking change.** `Node::set_attribute` accepts only `Attribute`
  or tuple with attribute name and value now.
- `Attributes` methods: `insert`, `remove` and `retain` will panic on an invalid
  input in debug mode now.
- `postproc::resolve_stop_attributes` no longer converts `offset` attribute
  into a `Number` type, leaving it in a `Length` type.
- **Breaking change.** This methods are require `&mut self` now:
  - `Document`: create_element, create_node, append, drain.
  - `Node`: detach, remove, drain, append, prepend, insert_after,
    insert_before, text_mut, set_text, set_id, set_tag_name,
    attributes_mut, set_attribute, set_attribute_checked, remove_attribute,
    remove_attributes
  - `postproc`: fix_rect_attributes, fix_poly_attributes
- Default numeric precision is 11 instead of 12 now.

### Removed
- `Node::attribute`. Use `node.attributes().get()` instead.
- `Node::attribute_value`. Use `node.attributes().get_value()` instead.
- `Node::has_attribute_with_value`.
- `Node::set_link_attribute`. Use `Node::set_attribute` instead.
- `Node::set_attribute_object`. Use `Node::set_attribute` instead.
- All `AttributeValue::as_*` methods.
- `Document::default`, because it was useless.

### Fixed
- `postproc::resolve_stop_attributes` can be executed multiple times without errors now.
- Additional whitespace after command in paths.

## [0.6.0] - 2017-06-18
### Added
- `Node::text_mut`.
- New text processing algorithm. Better `xml:space` support.

### Changed
- `postproc::resolve_inherit` doesn't return `Result` now.
  Any unresolved attributes will trigger a warning now.
- Node's text is stored as `String` and not as `Option<String>` now.
- `Node::text` returns `Ref<String>` now.
- Text will be preprocessed according to the
  [spec](https://www.w3.org/TR/SVG11/text.html#WhiteSpace) now.

### Removed
- `Error::UnresolvedAttribute`.

### Fixed
- Additional whitespace during ArcTo writing.

## [0.5.0] - 2017-06-05
### Added
- `postproc` module.
- `ElementType::is_gradient`.
- `Indent` enum instead of `i8` for `WriteOptions::indent`.
- Implemented `Display` trait for `path::Segment`.
- `path::Segment::fuzzy_eq`.
- `Transform::fuzzy_eq`.

### Changed
- All warnings will be printed to stderr now.
- `FromStream::from_data`, `Document::from_data`, `Document::from_data_with_opt`
  accepts `&str` instead of `&[u8]` now.
- Rename `FromStream::from_data` to `FromStream::from_str`.
- Rename `Document::from_data` to `Document::from_str`.
- Rename `Document::from_str_with_opt` to `Document::from_str_with_opt`.
- `Transform` uses default `PartialEq` implementation and not one with `FuzzyEq` now.
  Use `Transform::fuzzy_eq` method to get old result.

### Removed
- `Error::Utf8Error`, because `Document::from_data` accepts `&str` now.
- `WriteOptions::paths::coordinates_precision`.

## [0.4.0] - 2017-03-15
### Added
- `Node::make_copy` and `Node::make_deep_copy`.
- `Error::InvalidEncoding` and `Error::Utf8Error`.
- Input stream encoding validation.

### Changed
- `Node::prepend`, `Node::insert_after`, `Node::insert_before` accepts `&Node` now.

### Fixed
- Memory leak in `Document`. `Document` children were never deleted because of `Rc` crosslink.

## [0.3.1] - 2017-02-01
### Changed
- Use specific version of dependencies.

## [0.3.0] - 2017-01-14
### Added
- `AttributeValue::name`.
- `Length::zero`.
- `WriteOptions::paths::use_implicit_lineto_commands`.
- `WriteOptions::paths::coordinates_precision`.
- `FuzzyOrd` trait for `f64`.
- `Transform::apply_ref`.
- An external CSS parser, which brings support for universal and id selectors.
- Check that `style` element has a valid `type` attribute value.
- `parsing` build feature.
- `ElementType` trait for `Node`.
- `AttributeType` trait for `Attribute`.

### Changed
- Default numeric precision is 12 instead of 8 now.
- Float comparison is done using [float-cmp](https://crates.io/crates/float-cmp).
- `Node::set_attribute_object` now handles links.
- `Transform`'s `translate`, `scale`, `rotate`, `skew_x` and `skew_y` methods no longer
  consuming and modifies itself.
- `Node::is_referenced`, `Node::is_basic_shape` and `Node::is_container` moved
  to `ElementType` trait.
- Most of the `Attribute`'s `is_*` methods moved to `AttributeType` trait.
- Default transform matrices are not added to the DOM anymore.
- Empty list-based attributes are not added to the DOM anymore.

### Fixed
- `Transform`'s `rotate`, `skew_x` and `skew_y` methods doesn't worked correctly.

### Removed
- Custom SVG writer support and custom_writer example.
- `Attributes::get_value_or`. Use `Attributes::get_value().unwrap_or()` instead.

## [0.2.0] - 2016-11-04
### Added
- `Node::drain` method to remove nodes by the predicate without memory allocations.
- `Node::parents` - an iterator of `Node`s to the parents of a given node.
- Added support for implementing a custom SVG writer. See the `custom_writer` example for details.
- `Attributes::iter_svg_mut`.
- Default value for `clip` attribute.
- `path::Path::with_capacity` and `path::Builder::with_capacity`.
- `ParseOptions::skip_unresolved_classes`.

### Changed
- Always add a space after ArcTo flags during the path writing.
- SVG and non-SVG attributes now stored in the same container and not separately.
- Rename `LinkAttributes` to `LinkedNodes`.
- `descendants` and `children` methods now returns all nodes and not only SVG elements.
  Use the `svg()` method to get only SVG elements. Example: `descendants().svg()`.
- Rename `has_children_nodes` to `has_children`.
- The `LinkedNodes` iterator contains a reference to nodes vec and not a copy now.
  It will break node modifying while iterating. Less useful, but more correct.
- The `Document::create_element` accepts `&str` now.
- The `Document::create_element` will panic now if supplied string-based tag name is empty.
- The `Node::set_tag_name` will panic now if supplied string-based tag name is empty.
- The `write` module renamed to `writer` and made public.
- Attribute modules moved to `attribute` submodule. Doesn't impact API.

### Removed
- `descendant_nodes` and `children_nodes` methods.
- `Descendants::skip_children`.
- `ParseOptions::skip_svg_elements`.
- `Node::same_node`.
- `Document::create_nonsvg_element`. Use `Document::create_element` instead.
- `Node::set_tag_id`. Use `Node::set_tag_name` instead.
- `Node::is_tag_id`. Use `Node::is_tag_name` instead.
- `Node::has_child_with_tag_name`. Can be easily implemented with iterators.
- `Node::child_by_tag_name`. Can be easily implemented with iterators.
- `Node::child_by_tag_id`. Can be easily implemented with iterators.
- `Node::parent_element`. Can be easily implemented with iterators.
- `Node::parent_attribute`. Can be easily implemented with iterators.
- The `EmptyTagName` error type.

### Fixed
- `ParseOptions::parse_px_unit` now works in `LengthList`.
- CSS processing when style defined multiple times.

## [0.1.0] - 2016-10-09
### Added
- Missing license headers.
- The `children` method for the `Document`.
- The `is_inheritable` method for the `Attribute`.
- The `get_value_mut` method for the `Attributes`.
- `children_nodes`, `is_container`, `set_text`, methods for the `Node`.
- `has_translate`, `has_scale`, `has_proportional_scale`, `has_skew`, `has_rotate`, `get_translate`,
  `get_scale`, `get_skew`, `get_rotate`, `apply`, `rotate`, `skew_x`, `skew_y` methods to the `Transform`.
- `clip` and `font` attributes to the presentation attributes list.
- The `types::number::FuzzyEq` trait.
- A new error type: `EmptyTagName`.

### Changed
- More correct CSS2 processing.
- Rename `is_element` method into `is_svg_element` in the `Node`.
- Rename `to_absolute` method into `conv_to_absolute` in the `Path`.
- Rename `to_relative` method into `conv_to_relative` in the `Path`.
- Rename `descendants_all` method into `descendant_nodes` in the `Node`.
- Rename `get_or` method into `get_value_or` in the `Attributes`.
- The `children` method of the `Node` struct now returns an iterator over SVG elements and not all nodes.
  For all nodes you should use `children_nodes` method now.
- The `has_children` method now returns true if node has children elements, not nodes.
  For nodes you should use `has_children_nodes` method now.
- Remove redundant semicolon from error messages.
- We keep unknown attributes from styles now.
- Broken FuncIRI inside `fill` attributes now replaces with `none`.
- The `WriteOptions::numbers::remove_leading_zero` move to `WriteOptions::remove_leading_zero`.
- The `WriteOptions::transforms::simplify_matrix` move to `WriteOptions::simplify_transform_matrices`.
- Split the `Document::create_element` method into two: `create_element` and `create_nonsvg_element`.
- Split the `Node::set_tag_name` method into two: `set_tag_id` and `set_tag_name`.

### Fixed
- Attributes from ENTITY is now parsed and not inserted as is.
- `parse_unknown_attributes` flag doesn't processed correctly.
- ArcTo segment writing.

### Removed
- The `first_element_child` method from the `Document`. Use `doc.children().nth(0)` instead.
- `WriteOptions::numbers`. The precision is fixed now.
- The `find_reference_attribute` method from the `Node`.

## [0.0.3] - 2016-09-20
### Added
- A fallback value processing from the \<paint\> type.
- `has_attributes`, `remove`, `is_basic_shape`, `has_visible_attribute` methods to the `Node`.
- `is_graphical_event`, `is_conditional_processing`, `is_core`, `is_fill`, `is_stroke`,
  `is_animation_event`, `is_document_event`,   methods to the `Attribute`.
- `types::path::Segment` struct which is used instead of one from `libsvgparser`.
- `to_absolute` and `to_relative` methods to the `types::path::Path`.
- New error type: `BrokenFuncIri`.
- `is_*type*` methods to the `Attribute`. Like `is_number`, etc.

### Changed
- Moved back from `dtoa` to the std implementation.
- The `Transform` struct is now implements Copy.
- Nodes should be removed via `Node::remove` method and not via `Node::detach` + Drop.
- `Attributes` implemented using `Vec` and not `VecMap` now. It's much faster.
- Split `AttributeValue::Link` into `AttributeValue::Link` and `AttributeValue::FuncLink`.

### Fixed
- Fix crash in the NodeData's Drop.
- Fix attributes remove which contains links to removed node.
- Fix parsing of the empty `style` element.

## [0.0.2] - 2016-09-09
### Added
- `first_element_child`, `svg_element`, `create_element_with_text` methods to the `Document`.
- `has_parent`, `has_text_children`, `document` methods to the `Node`.

### Changed
- Use `dtoa::write()` instead of `write!()`.
- `Document::append` now returns added node.

### Fixed
- Fix default value of the 'stroke-miterlimit' attribute.
- Fix text generating when parent 'text' element has only one text node.

## 0.0.1 - 2016-08-26
### Added
- Initial release.

[Unreleased]: https://github.com/RazrFalcon/svgdom/compare/v0.14.0...HEAD
[0.14.0]: https://github.com/RazrFalcon/svgdom/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/RazrFalcon/svgdom/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/RazrFalcon/svgdom/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/RazrFalcon/svgdom/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/RazrFalcon/svgdom/compare/v0.10.5...v0.11.0
[0.10.5]: https://github.com/RazrFalcon/svgdom/compare/v0.10.4...v0.10.5
[0.10.4]: https://github.com/RazrFalcon/svgdom/compare/v0.10.3...v0.10.4
[0.10.3]: https://github.com/RazrFalcon/svgdom/compare/v0.10.2...v0.10.3
[0.10.2]: https://github.com/RazrFalcon/svgdom/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/RazrFalcon/svgdom/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/RazrFalcon/svgdom/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/RazrFalcon/svgdom/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/RazrFalcon/svgdom/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/RazrFalcon/svgdom/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/RazrFalcon/svgdom/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/RazrFalcon/svgdom/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/RazrFalcon/svgdom/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/RazrFalcon/svgdom/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/RazrFalcon/svgdom/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/RazrFalcon/svgdom/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/RazrFalcon/svgdom/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RazrFalcon/svgdom/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/RazrFalcon/svgdom/compare/0.0.3...v0.1.0
[0.0.3]: https://github.com/RazrFalcon/svgdom/compare/0.0.2...0.0.3
[0.0.2]: https://github.com/RazrFalcon/svgdom/compare/0.0.1...0.0.2
