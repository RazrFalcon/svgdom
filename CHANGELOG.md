# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) 
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- `first_element_child`, `svg_element`, `create_element_with_text` methods to `Document`.
- `has_parent`, `has_text_children`, `document` methods to `Node`.

### Changed
- Use `dtoa::write()` insted of `write!()`.
- `Document::append()` now returns added node.

### Fixed
- Fix default value of the 'stroke-miterlimit' attribute.
- Fix text generating when parent 'text' element has only one text node.

## 0.0.1 - 2016-08-26
### Added
- Initial release.

[Unreleased]: https://github.com/RazrFalcon/libsvgdom/compare/0.0.1...HEAD
