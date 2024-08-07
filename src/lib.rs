//! A platform agnostic Rust driver for the Sensirion SGP30 gas sensor, based
//! on the [`embedded-hal`](https://github.com/japaric/embedded-hal) traits.
//!
//! ## The Device
//!
//! The Sensirion SGP30 is a low-power gas sensor for indoor air quality
//! applications with good long-term stability. It has an I²C interface with TVOC
//! (*Total Volatile Organic Compounds*) and CO₂ equivalent signals.
//!
//! - [Datasheet](https://www.sensirion.com/file/datasheet_sgp30)
//! - [Product Page](https://www.sensirion.com/sgp)
//!
//! ## Usage
//!
//! ### Instantiating
//!
//! Import this crate and an `embedded_hal` implementation, then instantiate
//! the device:
//!
//! ```no_run
//! use linux_embedded_hal as hal;
//!
//! use hal::{Delay, I2cdev};
//! use sgp30::Sgp30;
//!
//! # fn main() {
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = 0x58;
//! let mut sgp = Sgp30::new(dev, address, Delay);
//! # }
//! ```
//!
//! ### Fetching Device Information
//!
//! You can fetch the serial number of your sensor as well as the [feature
//! set](struct.FeatureSet.html):
//!
//! ```no_run
//! # use linux_embedded_hal as hal;
//! # use hal::{Delay, I2cdev};
//! # use sgp30::Sgp30;
//! use sgp30::FeatureSet;
//!
//! # fn main() {
//! # let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! # let mut sgp = Sgp30::new(dev, 0x58, Delay);
//! let serial_number: [u8; 6] = sgp.serial().unwrap();
//! let feature_set: FeatureSet = sgp.get_feature_set().unwrap();
//! # }
//! ```
//!
//! ### Doing Measurements
//!
//! Before you do any measurements, you need to initialize the sensor.
//!
//! ```no_run
//! # use linux_embedded_hal as hal;
//! # use hal::{Delay, I2cdev};
//! # use sgp30::Sgp30;
//! # fn main() {
//! # let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! # let mut sgp = Sgp30::new(dev, 0x58, Delay);
//! sgp.init().unwrap();
//! # }
//! ```
//!
//! The SGP30 uses a dynamic baseline compensation algorithm and on-chip
//! calibration parameters to provide two complementary air quality signals.
//! Calling this method starts the air quality measurement. **After
//! initializing the measurement, the `measure()` method must be called in
//! regular intervals of 1 second** to ensure proper operation of the dynamic
//! baseline compensation algorithm. It is the responsibility of the user of
//! this driver to ensure that these periodic measurements are being done!
//!
//! ```no_run
//! # use linux_embedded_hal as hal;
//! # use hal::I2cdev;
//! # use sgp30::Sgp30;
//! use embedded_hal::delay::DelayNs;
//! use hal::Delay;
//! use sgp30::Measurement;
//!
//! # fn main() {
//! # let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! # let mut sgp = Sgp30::new(dev, 0x58, Delay);
//! # sgp.init().unwrap();
//! loop {
//!     let measurement: Measurement = sgp.measure().unwrap();
//!     println!("CO₂eq parts per million: {}", measurement.co2eq_ppm);
//!     println!("TVOC parts per billion: {}", measurement.tvoc_ppb);
//!     Delay.delay_ms(1000 - 12);
//! }
//! # }
//! ```
//!
//! *(Note: In the example we're using a delay of 988 ms because the
//! measurement takes up to 12 ms according to the datasheet. In reality, it
//! would be better to use a timer-based approach instead.)*
//!
//! For the first 15 s after initializing the air quality measurement, the
//! sensor is in an initialization phase during which it returns fixed
//! values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
//! the values should start to change.
//!
//! A new init command has to be sent after every power-up or soft reset.
//!
//! ### Restoring Baseline Values
//!
//! The SGP30 provides the possibility to read and write the values of the
//! baseline correction algorithm. This feature is used to save the baseline in
//! regular intervals on an external non-volatile memory and restore it after a
//! new power-up or soft reset of the sensor.
//!
//! The [`get_baseline()`](struct.Sgp30.html#method.get_baseline) method
//! returns the baseline values for the two air quality signals. After a
//! power-up or soft reset, the baseline of the baseline correction algorithm
//! can be restored by calling [`init()`](struct.Sgp30.html#method.init)
//! followed by [`set_baseline()`](struct.Sgp30.html#method.set_baseline).
//!
//! ```no_run
//! # use linux_embedded_hal as hal;
//! # use hal::{I2cdev, Delay};
//! # use sgp30::Sgp30;
//! use sgp30::Baseline;
//!
//! # fn main() {
//! # let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! # let mut sgp = Sgp30::new(dev, 0x58, Delay);
//! # sgp.init().unwrap();
//! let baseline: Baseline = sgp.get_baseline().unwrap();
//! // …
//! sgp.init().unwrap();
//! sgp.set_baseline(&baseline).unwrap();
//! # }
//! ```
//!
//! ### Humidity Compensation
//!
//! The SGP30 features an on-chip humidity compensation for the air quality
//! signals (CO₂eq and TVOC) and sensor raw signals (H2 and Ethanol). To use
//! the on-chip humidity compensation, an absolute humidity value from an
//! external humidity sensor is required.
//!
//! ```no_run
//! # use linux_embedded_hal as hal;
//! # use hal::{I2cdev, Delay};
//! # use sgp30::Sgp30;
//! use sgp30::Humidity;
//!
//! # fn main() {
//! # let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! # let mut sgp = Sgp30::new(dev, 0x58, Delay);
//! // This value must be obtained from a separate humidity sensor
//! let humidity = Humidity::from_f32(23.42).unwrap();
//!
//! sgp.init().unwrap();
//! sgp.set_humidity(Some(&humidity)).unwrap();
//! # }
//! ```
//!
//! After setting a new humidity value, this value will be used by the
//! on-chip humidity compensation algorithm until a new humidity value is
//! set. Restarting the sensor (power-on or soft reset) or calling the
//! function with a `None` value sets the humidity value used for
//! compensation to its default value (11.57 g/m³) until a new humidity
//! value is sent.
//!
//! ## `embedded-hal-async` support
//!
//! This crate has optional support for the [`embedded-hal-async`] crate, which
//! provides `async` versions of the `I2c` and `DelayNs` traits. Async support
//! is an off-by-default optional feature, so that projects which aren't using
//! [`embedded-hal-async`] can avoid the additional dependency.
//!
//! To use this crate with `embedded-hal-async`, enable the `embedded-hal-async`
//! feature flag in your `Cargo.toml`:
//!
//! ```toml
//! sgp30 = { version = "1", features = ["embedded-hal-async"] }
//! ```
//!
//! Once the `embedded-hal-async` feature is enabled, construct an instance of
//! the [`Sgp30Async`] struct, providing types implementing the
//! [`embedded_hal_async::i2c::I2c`] and [`embedded_hal_async::delay::DelayNs`]
//! traits. The [`Sgp30Async`] struct is identical to the [`Sgp30`] struct,
//! except that its methods are `async fn`s.
//!
//! [`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
//! [`embedded_hal_async::i2c::I2c`]: https://docs.rs/embedded-hal-async/embedded-hal-async

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use byteorder::{BigEndian, ByteOrder};
use embedded_hal as hal;
use sensirion_i2c::{crc8, i2c};

use crate::hal::{
    delay::DelayNs,
    i2c::{ErrorType, I2c},
};

#[cfg(feature = "embedded-hal-async")]
mod async_impl;
#[cfg(feature = "embedded-hal-async")]
pub use async_impl::Sgp30Async;

mod types;

pub use crate::types::{Baseline, FeatureSet, Humidity, Measurement, ProductType, RawSignals};

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error during a write
    I2cWrite(E),
    /// I²C bus error during a read
    I2cRead(E),
    /// CRC checksum validation failed
    Crc,
    /// User tried to measure the air quality without starting the
    /// initialization phase.
    NotInitialized,
}

impl<I> From<i2c::Error<I>> for Error<I::Error>
where
    I: ErrorType,
{
    fn from(err: i2c::Error<I>) -> Self {
        match err {
            i2c::Error::Crc => Error::Crc,
            i2c::Error::I2cWrite(e) => Error::I2cRead(e),
            i2c::Error::I2cRead(e) => Error::I2cWrite(e),
        }
    }
}

/// I²C commands sent to the sensor.
#[derive(Debug, Copy, Clone)]
enum Command {
    /// Return the serial number.
    GetSerial,
    /// Run an on-chip self-test.
    SelfTest,
    /// Initialize air quality measurements.
    InitAirQuality,
    /// Get a current air quality measurement.
    MeasureAirQuality,
    /// Measure raw signals.
    MeasureRawSignals,
    /// Return the baseline value.
    GetBaseline,
    /// Set the baseline value.
    SetBaseline,
    /// Set the current absolute humidity.
    SetHumidity,
    /// Set the feature set.
    GetFeatureSet,
}

impl Command {
    fn as_bytes(self) -> [u8; 2] {
        match self {
            Command::GetSerial => [0x36, 0x82],
            Command::SelfTest => [0x20, 0x32],
            Command::InitAirQuality => [0x20, 0x03],
            Command::MeasureAirQuality => [0x20, 0x08],
            Command::MeasureRawSignals => [0x20, 0x50],
            Command::GetBaseline => [0x20, 0x15],
            Command::SetBaseline => [0x20, 0x1E],
            Command::SetHumidity => [0x20, 0x61],
            Command::GetFeatureSet => [0x20, 0x2F],
        }
    }

    /// Writes this command and the provided `data` bytes to `buf`, returning a
    /// slice of the written portion of `buf`.
    ///
    /// # Arguments
    ///
    /// - `buf`: The buffer into which to write the command and data bytes.
    ///   This buffer must be 8 bytes long.
    /// - `data`: The data bytes to write after the command bytes. This slice
    ///   must contain either 2 or 4 bytes.
    ///
    /// # Panics
    ///
    /// - If `data` is not either 2 or 4 bytes long.
    fn as_bytes_with_data<'buf>(self, buf: &'buf mut [u8; 8], data: &[u8]) -> &'buf [u8] {
        assert!(data.len() == 2 || data.len() == 4);
        buf[0..2].copy_from_slice(&self.as_bytes());
        buf[2..4].copy_from_slice(&data[0..2]);
        buf[4] = crc8::calculate(&data[0..2]);
        if data.len() > 2 {
            buf[5..7].copy_from_slice(&data[2..4]);
            buf[7] = crc8::calculate(&data[2..4]);
        }
        if data.len() > 2 {
            &buf[0..8]
        } else {
            &buf[0..5]
        }
    }
}

/// Driver for the SGP30
#[derive(Debug, Default)]
pub struct Sgp30<I2C, D> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The I²C device address.
    address: u8,
    /// The concrete Delay implementation.
    delay: D,
    /// Whether the air quality measurement was initialized.
    initialized: bool,
}

/// The fixed data pattern returned when the on-chip self-test is successful.
const SELFTEST_SUCCESS: &[u8] = &[0xd4, 0x00];

impl<I2C, D> Sgp30<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    /// Create a new instance of the SGP30 driver.
    pub fn new(i2c: I2C, address: u8, delay: D) -> Self {
        Sgp30 {
            i2c,
            address,
            delay,
            initialized: false,
        }
    }

    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Write an I²C command to the sensor.
    fn send_command(&mut self, command: Command) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &command.as_bytes())
            .map_err(Error::I2cWrite)
    }

    /// Write an I²C command and data to the sensor.
    ///
    /// The data slice must have a length of 2 or 4.
    ///
    /// CRC checksums will automatically be added to the data.
    fn send_command_and_data(
        &mut self,
        command: Command,
        data: &[u8],
    ) -> Result<(), Error<I2C::Error>> {
        let mut buf = [0; 2 /* command */ + 6 /* max length of data + crc */];
        let payload = command.as_bytes_with_data(&mut buf, data);
        self.i2c
            .write(self.address, payload)
            .map_err(Error::I2cWrite)
    }

    /// Return the 48 bit serial number of the SGP30.
    pub fn serial(&mut self) -> Result<[u8; 6], Error<I2C::Error>> {
        // Request serial number
        self.send_command(Command::GetSerial)?;

        // Recommended wait time according to datasheet (6.5)
        self.delay.delay_us(500);

        // Read serial number
        let mut buf = [0; 9];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;

        Ok([buf[0], buf[1], buf[3], buf[4], buf[6], buf[7]])
    }

    /// Run an on-chip self-test. Return a boolean indicating whether the test succeeded.
    pub fn selftest(&mut self) -> Result<bool, Error<I2C::Error>> {
        // Start self test
        self.send_command(Command::SelfTest)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(220);

        // Read result
        let mut buf = [0; 3];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;

        // Compare with self-test success pattern
        Ok(&buf[0..2] == SELFTEST_SUCCESS)
    }

    /// Initialize the air quality measurement.
    ///
    /// The SGP30 uses a dynamic baseline compensation algorithm and on-chip
    /// calibration parameters to provide two complementary air quality
    /// signals.
    ///
    /// Calling this method starts the air quality measurement. After
    /// initializing the measurement, the `measure()` method must be called in
    /// regular intervals of 1 s to ensure proper operation of the dynamic
    /// baseline compensation algorithm. It is the responsibility of the user
    /// of this driver to ensure that these periodic measurements are being
    /// done.
    ///
    /// For the first 15 s after initializing the air quality measurement, the
    /// sensor is in an initialization phase during which it returns fixed
    /// values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
    /// the values should start to change.
    ///
    /// A new init command has to be sent after every power-up or soft reset.
    pub fn init(&mut self) -> Result<(), Error<I2C::Error>> {
        if self.initialized {
            // Already initialized
            return Ok(());
        }
        self.force_init()
    }

    /// Like [`init()`](struct.Sgp30.html#method.init), but without checking
    /// whether the sensor is already initialized.
    ///
    /// This might be necessary after a sensor soft or hard reset.
    pub fn force_init(&mut self) -> Result<(), Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::InitAirQuality)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10);

        self.initialized = true;
        Ok(())
    }

    /// Get an air quality measurement.
    ///
    /// Before calling this method, the air quality measurements must have been
    /// initialized using the [`init()`](struct.Sgp30.html#method.init) method.
    /// Otherwise an [`Error::NotInitialized`](enum.Error.html#variant.NotInitialized)
    /// will be returned.
    ///
    /// Once the measurements have been initialized, the
    /// [`measure()`](struct.Sgp30.html#method.measure) method must be called
    /// in regular intervals of 1 s to ensure proper operation of the dynamic
    /// baseline compensation algorithm. It is the responsibility of the user
    /// of this driver to ensure that these periodic measurements are being
    /// done.
    ///
    /// For the first 15 s after initializing the air quality measurement, the
    /// sensor is in an initialization phase during which it returns fixed
    /// values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
    /// the values should start to change.
    pub fn measure(&mut self) -> Result<Measurement, Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command to sensor
        self.send_command(Command::MeasureAirQuality)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(12);

        // Read result
        let mut buf = [0; 6];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;
        Ok(Measurement::from_bytes(&buf))
    }

    /// Return sensor raw signals.
    ///
    /// This command is intended for part verification and testing purposes. It
    /// returns the raw signals which are used as inputs for the on-chip
    /// calibration and baseline compensation algorithm. The command performs a
    /// measurement to which the sensor responds with the two signals for H2
    /// and Ethanol.
    pub fn measure_raw_signals(&mut self) -> Result<RawSignals, Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command to sensor
        self.send_command(Command::MeasureRawSignals)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(25);

        // Read result
        let mut buf = [0; 6];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;
        Ok(RawSignals::from_bytes(&buf))
    }

    /// Return the baseline values of the baseline correction algorithm.
    ///
    /// The SGP30 provides the possibility to read and write the baseline
    /// values of the baseline correction algorithm. This feature is used to
    /// save the baseline in regular intervals on an external non-volatile
    /// memory and restore it after a new power-up or soft reset of the sensor.
    ///
    /// This function returns the baseline values for the two air quality
    /// signals. These two values should be stored on an external memory. After
    /// a power-up or soft reset, the baseline of the baseline correction
    /// algorithm can be restored by calling
    /// [`init()`](struct.Sgp30.html#method.init) followed by
    /// [`set_baseline()`](struct.Sgp30.html#method.set_baseline).
    pub fn get_baseline(&mut self) -> Result<Baseline, Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::GetBaseline)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10);

        // Read result
        let mut buf = [0; 6];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;
        Ok(Baseline::from_bytes(&buf))
    }

    /// Set the baseline values for the baseline correction algorithm.
    ///
    /// Before calling this method, the air quality measurements must have been
    /// initialized using the [`init()`](struct.Sgp30.html#method.init) method.
    /// Otherwise an [`Error::NotInitialized`](enum.Error.html#variant.NotInitialized)
    /// will be returned.
    ///
    /// The SGP30 provides the possibility to read and write the baseline
    /// values of the baseline correction algorithm. This feature is used to
    /// save the baseline in regular intervals on an external non-volatile
    /// memory and restore it after a new power-up or soft reset of the sensor.
    ///
    /// This function sets the baseline values for the two air quality
    /// signals.
    pub fn set_baseline(&mut self, baseline: &Baseline) -> Result<(), Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command and data to sensor
        // Note that the order of the two parameters is inverted when writing
        // compared to when reading.
        let mut buf = [0; 4];
        BigEndian::write_u16(&mut buf[0..2], baseline.tvoc);
        BigEndian::write_u16(&mut buf[2..4], baseline.co2eq);
        self.send_command_and_data(Command::SetBaseline, &buf)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10);

        Ok(())
    }

    /// Set the humidity value for the baseline correction algorithm.
    ///
    /// The SGP30 features an on-chip humidity compensation for the air quality
    /// signals (CO₂eq and TVOC) and sensor raw signals (H2 and Ethanol). To
    /// use the on-chip humidity compensation, an absolute humidity value from
    /// an external humidity sensor is required.
    ///
    /// After setting a new humidity value, this value will be used by the
    /// on-chip humidity compensation algorithm until a new humidity value is
    /// set. Restarting the sensor (power-on or soft reset) or calling the
    /// function with a `None` value sets the humidity value used for
    /// compensation to its default value (11.57 g/m³) until a new humidity
    /// value is sent.
    ///
    /// Before calling this method, the air quality measurements must have been
    /// initialized using the [`init()`](struct.Sgp30.html#method.init) method.
    /// Otherwise an [`Error::NotInitialized`](enum.Error.html#variant.NotInitialized)
    /// will be returned.
    pub fn set_humidity(&mut self, humidity: Option<&Humidity>) -> Result<(), Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command and data to sensor
        let buf = match humidity {
            Some(humi) => humi.as_bytes(),
            None => [0, 0],
        };
        self.send_command_and_data(Command::SetHumidity, &buf)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10);

        Ok(())
    }

    /// Get the feature set.
    ///
    /// The SGP30 features a versioning system for the available set of
    /// measurement commands and on-chip algorithms. This so called feature set
    /// version number can be read out with this method.
    pub fn get_feature_set(&mut self) -> Result<FeatureSet, Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::GetFeatureSet)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(2);

        // Read result
        let mut buf = [0; 3];
        i2c::read_words_with_crc(&mut self.i2c, self.address, &mut buf)?;

        Ok(FeatureSet::parse(buf[0], buf[1]))
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock as hal;

    use self::hal::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction},
    };
    use super::*;

    /// Test the `serial` function
    #[test]
    fn serial() {
        let expectations = [
            Transaction::write(0x58, Command::GetSerial.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0, 0, 129, 0, 100, 254, 204, 130, 135]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        let serial = sgp.serial().unwrap();
        assert_eq!(serial, [0, 0, 0, 100, 204, 130]);
        sgp.destroy().done();
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_ok() {
        let expectations = [
            Transaction::write(0x58, Command::SelfTest.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0xD4, 0x00, 0xC6]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        assert!(sgp.selftest().unwrap());
        sgp.destroy().done();
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_fail() {
        let expectations = [
            Transaction::write(0x58, Command::SelfTest.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0x12, 0x34, 0x37]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        assert!(!sgp.selftest().unwrap());
        sgp.destroy().done();
    }

    /// Test the `measure` function: Require initialization
    #[test]
    fn measure_initialization_required() {
        let mock = I2cMock::new(&[]);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        match sgp.measure() {
            Err(Error::NotInitialized) => {}
            Ok(_) => panic!("Error::NotInitialized not returned"),
            Err(_) => panic!("Wrong error returned"),
        }
        sgp.destroy().done();
    }

    /// Test the `measure` function: Calculation of return values
    #[test]
    fn measure_success() {
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, Command::MeasureAirQuality.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let measurements = sgp.measure().unwrap();
        assert_eq!(measurements.co2eq_ppm, 4_660);
        assert_eq!(measurements.tvoc_ppb, 54_274);
        sgp.destroy().done();
    }

    /// Test the `get_baseline` function
    #[test]
    fn get_baseline() {
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, Command::GetBaseline.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let baseline = sgp.get_baseline().unwrap();
        assert_eq!(baseline.co2eq, 4_660);
        assert_eq!(baseline.tvoc, 54_274);
        sgp.destroy().done();
    }

    /// Test the `set_baseline` function
    #[test]
    fn set_baseline() {
        #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x1E,
                /* data + crc8: */ 0x56, 0x78, 0x7D, 0x12, 0x34, 0x37,
            ]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let baseline = Baseline {
            co2eq: 0x1234,
            tvoc: 0x5678,
        };
        sgp.set_baseline(&baseline).unwrap();
        sgp.destroy().done();
    }

    /// Test the `set_humidity` function
    #[test]
    fn set_humidity() {
        #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x61,
                /* data + crc8: */ 0x0F, 0x80, 0x62,
            ]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let humidity = Humidity::from_f32(15.5).unwrap();
        sgp.set_humidity(Some(&humidity)).unwrap();
        sgp.destroy().done();
    }

    /// Test the `set_humidity` function with a None value
    #[test]
    fn set_humidity_none() {
        #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x61,
                /* data + crc8: */ 0x00, 0x00, 0x81,
            ]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        sgp.set_humidity(None).unwrap();
        sgp.destroy().done();
    }

    /// Test the `get_feature_set` function.
    #[test]
    fn get_feature_set() {
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, Command::GetFeatureSet.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0x00, 0x42, 0xDE]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let feature_set = sgp.get_feature_set().unwrap();
        assert_eq!(feature_set.product_type, ProductType::Sgp30);
        assert_eq!(feature_set.product_version, 0x42);
        sgp.destroy().done();
    }

    /// Test the `measure_raw_signals` function.
    #[test]
    fn measure_raw_signals() {
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, Command::MeasureRawSignals.as_bytes()[..].into()),
            Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0x56, 0x78, 0x7D]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sgp = Sgp30::new(mock, 0x58, NoopDelay);
        sgp.init().unwrap();
        let signals = sgp.measure_raw_signals().unwrap();
        assert_eq!(signals.h2, (0x12 << 8) + 0x34);
        assert_eq!(signals.ethanol, (0x56 << 8) + 0x78);
        sgp.destroy().done();
    }
}
