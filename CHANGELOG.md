# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) 
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- A fallback value processing from the \<paint\> type.
- 'has_attributes()' method to the `Node`.

### Changed
- Moved back from 'dtoa' to the std implementation.
- The 'Transfrom' struct is now implements Copy.

### Fixed
- Fix crash in the NodeData's Drop.

## [0.0.2] - 2016-09-09
### Added
- `first_element_child`, `svg_element`, `create_element_with_text` methods to the `Document`.
- `has_parent`, `has_text_children`, `document` methods to the `Node`.

### Changed
- Use `dtoa::write()` insted of `write!()`.
- `Document::append()` now returns added node.

### Fixed
- Fix default value of the 'stroke-miterlimit' attribute.
- Fix text generating when parent 'text' element has only one text node.

## 0.0.1 - 2016-08-26
### Added
- Initial release.

[Unreleased]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.2...HEAD
[0.0.2]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.1...0.0.2
