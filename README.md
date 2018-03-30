# Rust SGP30 Driver

[![CircleCI][circle-ci-badge]][circle-ci]
[![Crates.io Version][crates-io-badge]][crates-io]
[![Crates.io Downloads][crates-io-download-badge]][crates-io-download]

This is a platform agnostic Rust driver for the Sensirion SGP30, based on the
[`embedded-hal`](https://github.com/japaric/embedded-hal) traits.

## The Device

The Sensirion SGP30 is a low-power gas sensor for indoor air quality
applications with good long-term stability. It has an I²C interface with TVOC
(*Total Volatile Organic Compounds*) and CO₂ equivalent signals.

Datasheet: https://www.sensirion.com/file/datasheet_sgp30

## Status

- [x] Measure air quality
- [x] Get and set baseline
- [ ] Set humidity
- [ ] Get feature set version
- [ ] Get raw signals
- [x] Get serial number
- [x] Support on-chip self-test
- [x] CRC checks
- [ ] Docs
- [ ] Publish to crates.io

## Linting

To run clippy lints, compile the library with `--features clippy` on a nightly
compiler:

    $ cargo build --features clippy

<!-- Badges -->
[circle-ci]: https://circleci.com/gh/dbrgn/sgp30-rs/tree/master
[circle-ci-badge]: https://circleci.com/gh/dbrgn/sgp30-rs/tree/master.svg?style=shield
[crates-io]: https://crates.io/crates/sgp30
[crates-io-badge]: https://img.shields.io/crates/v/sgp30.svg?maxAge=3600
[crates-io-download]: https://crates.io/crates/sgp30
[crates-io-download-badge]: https://img.shields.io/crates/d/sgp30.svg?maxAge=3600
