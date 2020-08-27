# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [0.3.1] - 2020-08-28

### Changed

- Use the [sensirion-i2c](https://crates.io/crates/sensirion-i2c) crate for a
  more simplified codebase


## [0.3.0] - 2020-08-17

### Changed

- Convert to Rust 2018, now requires at least Rust 1.32


## [0.2.1] - 2018-12-02

### Fixed

- Fix order of values when writing baseline ([#5][i5], thanks @slim-bean)


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


[Unreleased]: https://github.com/dbrgn/sgp30-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/dbrgn/sgp30-rs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.0...v0.1.1

[i5]: https://github.com/dbrgn/sgp30-rs/pull/5
