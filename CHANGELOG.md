# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Missing license headers.
- The `children` method for the `Document`.
- The `children_nodes` method for the `Node`.
- `has_translate`, `has_scale`, `has_proportional_scale`, `has_skew`, `has_rotate`, `get_translate`,
  `get_scale`, `get_skew`, `get_rotate`, `apply` methods to the `Transform`.

### Changed
- Rename `is_element` method into `is_svg_element` in the `Node`.
- Rename `to_absolute` method into `conv_to_absolute` in the `Path`.
- Rename `to_relative` method into `conv_to_relative` in the `Path`.
- Rename `descendants_all` method into `descendant_nodes` in the `Node`.
- The `children` method of the `Node` struct now returns an iterator over SVG elements and not all nodes.
  For all nodes you should use `children_nodes` method now.
- The `has_children` method now returns true if node has children elements, not nodes.
  For nodes you should use `has_children_nodes` method now.

- Remove redundant semicolon from error messages.

### Fixed
- Attributes from ENTITY is now parsed and not inserted as is.

### Removed
- `first_element_child` method from the `Document`. Use `doc.children().nth(0)` instead.

## [0.0.3] - 2016-09-20
### Added
- A fallback value processing from the \<paint\> type.
- `has_attributes`, `remove`, `is_basic_shape`, `has_visible_attribute` methods to the `Node`.
- `is_graphical_event`, `is_conditional_processing`, `is_core`, `is_fill`, `is_stroke`, `is_animation_event`, `is_document_event`,   methods to the `Attribute`.
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
- Use `dtoa::write()` insted of `write!()`.
- `Document::append` now returns added node.

### Fixed
- Fix default value of the 'stroke-miterlimit' attribute.
- Fix text generating when parent 'text' element has only one text node.

## 0.0.1 - 2016-08-26
### Added
- Initial release.

[Unreleased]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.3...HEAD
[0.0.3]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.2...0.0.3
[0.0.2]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.1...0.0.2
