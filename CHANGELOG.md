# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [1.0.0] - 2024-10-15

Identical to version 1.0.0-rc.1.


## [1.0.0-rc.1] - 2024-07-21

This release upgrades to `embedded-hal` 1.0 and also adds async support.

To use the async API, you must enable the `embedded-hal-async` Cargo feature.

### Added

- Support for async through `embedded-hal-async` (#19)

### Changed

- Upgrade to `embedded-hal` 1.x (#18)
- Bump MSRV to 1.75 (#22)
- Switch to Rust 2021 edition (#23)


## [0.3.2] - 2023-12-26

This is a maintenance release without any relevant changes to consumers of this
driver.

### Changed

- Update the [sensirion-i2c](https://crates.io/crates/sensirion-i2c) dependency to 0.2


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


[Unreleased]: https://github.com/dbrgn/sgp30-rs/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/dbrgn/sgp30-rs/compare/v1.0.0-rc.1...v1.0.0
[1.0.0-rc.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.3.2...v1.0.0-rc.1
[0.3.2]: https://github.com/dbrgn/sgp30-rs/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/dbrgn/sgp30-rs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dbrgn/sgp30-rs/compare/v0.1.0...v0.1.1

[i5]: https://github.com/dbrgn/sgp30-rs/pull/5
