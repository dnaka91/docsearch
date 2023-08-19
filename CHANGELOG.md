<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

## [0.3.4] - 2023-08-19

- Pin `serde` version to avoid using pre-compiled binaries during the build process.

## [0.3.3] - 2023-07-29

### Fixed

- Adjust to changes in the latest index format.

### Refactoring

- Replace the `chumsky` parser with `winnow` for a more lightweight, faster and easier to understand parsing logic of the v1 index format.

## [0.3.2] - 2022-06-03

### Fixed

- Correct index v3 detection for latest changes in nightly.

## [0.3.1] - 2022-05-30

### Changed

- Replace `unicode-xid` with `unicode-ident` for faster parsing of identifiers (and less memory usage too).

## [0.3.0] - 2022-01-26

### Changed

- Update to Rust edition 2021.
- Bump minimum Rust version to `1.58`.

### Fixed

- Correct URL retrieval as the docs.rs redirect logic changed.
- Adjust to the new data tags on docs.rs to retrieve the search index file.

## [0.2.0] - 2021-07-30

### Added

- Paths that only contain the crate name with no item (like just `anyhow` or `std`), will now resolve to the base crate docs instead of failing to find a link.

### Changed

- Complete parsing logic for FQNs to match with the Rust spec.
- Rename the `Fqn` type to `SimplePath` to have the same naming as the Rust spec.
- Expose all fields of the `Index` struct as the data can be changed through `serde` anyways.

## [0.1.1] - 2021-07-30

### Changed

- Derive `PartialEq`, `Eq` and `Hash` on the `Index` struct to make it hashable.
- Derive `serde::Serialize` and `serde::Deserialize` on the `Index` struct.

## [0.1.0]

### Added

- Initial release.

[Unreleased]: https://github.com/dnaka91/docsearch/compare/v0.3.4...HEAD
[0.3.4]: https://github.com/dnaka91/docsearch/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/dnaka91/docsearch/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/dnaka91/docsearch/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/dnaka91/docsearch/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/dnaka91/docsearch/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/dnaka91/docsearch/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dnaka91/docsearch/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dnaka91/docsearch/releases/tag/v0.1.0
