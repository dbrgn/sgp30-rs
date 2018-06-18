# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

...

## [0.2.0] - 2018-06-18

### Fixed

- Reexport `types::ProductType`

## [0.1.1] - 2018-04-01

### Changed

- The `Command` enum is not pub anymore

### Fixed

- The crate did not compile on `no_std`, because of a dependency and because of
  std methods on floats. This is now fixed by using the num-traits crate.

## 0.1.0 - 2018-03-31

This is the initial release to crates.io of the feature-complete driver. There
may be some API changes in the future, in case I decide that something can be
further improved. All changes will be documented in this CHANGELOG.

[Unreleased]: https://github.com/dbrgn/sgp30-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.0...v0.1.1
