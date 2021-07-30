<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

## [0.1.1] - 2021-07-30

### Changed

- Derive `PartialEq`, `Eq` and `Hash` on the `Index` struct to make it hashable.
- Derive `serde::Serialize` and `serde::Deserialize` on the `Index` struct.

## [0.1.0]

### Added

- Initial release.

[Unreleased]: https://github.com/dnaka91/docsearch/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/dnaka91/docsearch/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dnaka91/docsearch/releases/tag/v0.1.0
