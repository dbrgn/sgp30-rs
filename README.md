# Rust SGP30 Driver

[![Build status][workflow-badge]][workflow]
[![Crates.io Version][crates-io-badge]][crates-io]
[![Crates.io Downloads][crates-io-download-badge]][crates-io-download]
![No Std][no-std-badge]

This is a platform agnostic Rust driver for the Sensirion SGP30 gas sensor,
based on the [`embedded-hal`](https://github.com/japaric/embedded-hal) or
[`embedded-hal-async`] traits.

Docs: https://docs.rs/sgp30

## The Device

The Sensirion SGP30 is a low-power gas sensor for indoor air quality
applications with good long-term stability. It has an I²C interface with TVOC
(*Total Volatile Organic Compounds*) and CO₂ equivalent signals.

Datasheet: https://www.sensirion.com/file/datasheet_sgp30

## Status

- [x] Measure air quality
- [x] Get and set baseline
- [x] Set humidity
- [x] Get feature set
- [x] Get raw signals
- [x] Get serial number
- [x] Support on-chip self-test
- [x] CRC checks
- [x] Docs
- [x] [`embedded-hal-async] support

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
<!-- Badges -->
[workflow]: https://github.com/dbrgn/sgp30-rs/actions?query=workflow%3ACI
[workflow-badge]: https://github.com/dbrgn/sgp30-rs/actions/workflows/ci.yml/badge.svg
[crates-io]: https://crates.io/crates/sgp30
[crates-io-badge]: https://img.shields.io/crates/v/sgp30.svg?maxAge=3600
[crates-io-download]: https://crates.io/crates/sgp30
[crates-io-download-badge]: https://img.shields.io/crates/d/sgp30.svg?maxAge=3600
[no-std-badge]: https://img.shields.io/badge/no__std-yes-blue
