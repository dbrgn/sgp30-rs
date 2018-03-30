//! SGP30

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

extern crate embedded_hal as hal;

use hal::blocking::delay::{DelayMs, DelayUs};
use hal::blocking::i2c::{Read, Write, WriteRead};


const CRC8_POLYNOMIAL: u8 = 0x31;


/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2c(E),
    /// CRC checksum validation failed
    Crc,
    /// User tried to measure the air quality without starting the
    /// initialization phase.
    NotInitialized,
}


/// I²C commands sent to the sensor.
#[derive(Debug)]
pub enum Command {
    /// Return the serial number.
    GetSerial,
    /// Run an on-chip self-test.
    SelfTest,
    /// Initialize air quality measurements.
    InitAirQuality,
    /// Get a current air quality measurement.
    MeasureAirQuality,
    /// Return the baseline value.
    GetBaseline,
    /// Set the baseline value.
    SetBaseline,
    /// Set the current relative humidity.
    SetHumidity,
}

impl Command {
    fn as_bytes(&self) -> [u8; 2] {
        match *self {
            Command::GetSerial => [0x36, 0x82],
            Command::SelfTest => [0x20, 0x32],
            Command::InitAirQuality => [0x20, 0x03],
            Command::MeasureAirQuality => [0x20, 0x08],
            Command::GetBaseline => [0x20, 0x15],
            Command::SetBaseline => [0x20, 0x1E],
            Command::SetHumidity => [0x20, 0x61],
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

impl<I2C, D, E> Sgp30<I2C, D>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayUs<u16> + DelayMs<u16>,
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

    /// Write an I²C command to the sensor.
    fn send_command(&mut self, command: Command) -> Result<(), Error<E>> {
        self.i2c
            .write(self.address, &command.as_bytes())
            .map_err(Error::I2c)
    }

    /// Iterate over the provided buffer and validate the CRC8 checksum.
    ///
    /// If the checksum is wrong, return `Error::Crc`.
    ///
    /// Note: This method will consider every third byte a checksum byte. If
    /// the buffer size is not a multiple of 3, then not all data will be
    /// validated.
    fn validate_crc(&self, buf: &[u8]) -> Result<(), Error<E>> {
        for chunk in buf.chunks(3) {
            if chunk.len() == 3 {
                if crc8(&[chunk[0], chunk[1]]) != chunk[2] {
                    return Err(Error::Crc);
                }
            }
        }
        Ok(())
    }

    /// Read data into the provided buffer and validate the CRC8 checksum.
    ///
    /// If the checksum is wrong, return `Error::Crc`.
    ///
    /// Note: This method will consider every third byte a checksum byte. If
    /// the buffer size is not a multiple of 3, then not all data will be
    /// validated.
    fn read_with_crc(&mut self, mut buf: &mut [u8]) -> Result<(), Error<E>> {
        self.i2c
            .read(self.address, &mut buf)
            .map_err(Error::I2c)?;
        self.validate_crc(&buf)
    }

    /// Return the 48 bit serial number of the SGP30.
    pub fn serial(&mut self) -> Result<[u8; 6], Error<E>> {
        // Request serial number
        self.send_command(Command::GetSerial)?;

        // Recommended wait time according to datasheet (6.5)
        self.delay.delay_us(500);

        // Read serial number
        let mut buf = [0; 9];
        self.read_with_crc(&mut buf)?;

        Ok([
           buf[0], buf[1],
           buf[3], buf[4],
           buf[6], buf[7],
        ])
    }

    /// Run an on-chip self-test. Return a boolean indicating whether the test succeeded.
    pub fn selftest(&mut self) -> Result<bool, Error<E>> {
        // Start self test
        self.send_command(Command::SelfTest)?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(220);

        // Read result
        let mut buf = [0; 3];
        self.read_with_crc(&mut buf)?;

        // Compare with self-test success pattern
        Ok(&buf[0..2] == &[0xd4, 0x00])
    }

    /// Initialize the air quality measurement.
    ///
    /// The SGP30 uses a dynamic baseline compensation algorithm and on-chip
    /// calibration parameters to provide two complementary air quality
    /// signals.
    ///
    /// Calling this method starts the air quality measurement. After
    /// initializing the measurement, the `measure()` method must be called in
    /// regular intervals of 1s to ensure proper operation of the dynamic
    /// baseline compensation algorithm. It is the responsibility of the user
    /// of this driver to ensure that these periodic measurements are being
    /// done.
    ///
    /// For the first 15 s after initializing the air quality measurement, the
    /// sensor is in an initialization phase during which it returns fixed
    /// values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
    /// the values should start to change.
    pub fn init(&mut self) -> Result<(), Error<E>> {
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
    pub fn force_init(&mut self) -> Result<(), Error<E>> {
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
    /// in regular intervals of 1s to ensure proper operation of the dynamic
    /// baseline compensation algorithm. It is the responsibility of the user
    /// of this driver to ensure that these periodic measurements are being
    /// done.
    ///
    /// For the first 15 s after initializing the air quality measurement, the
    /// sensor is in an initialization phase during which it returns fixed
    /// values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
    /// the values should start to change.
    pub fn measure(&mut self) -> Result<(u16, u16), Error<E>> {
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
        self.read_with_crc(&mut buf)?;
        let co2eq_ppm = ((buf[0] as u16) << 8) | buf[1] as u16;
        let tvoc_ppb = ((buf[3] as u16) << 8) | buf[4] as u16;

        Ok((co2eq_ppm, tvoc_ppb))
    }
}

/// Calculate the CRC8 checksum.
///
/// Implementation based on the reference implementation by Sensirion.
fn crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0xff;
    for byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) > 0 {
                crc = (crc << 1) ^ CRC8_POLYNOMIAL;
            } else {
                crc = crc << 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    extern crate embedded_hal_mock as hal;
    use super::*;

    /// Test the crc8 function against the test value provided in the
    /// datasheet (section 6.6).
    #[test]
    fn crc8_test_value() {
        assert_eq!(crc8(&[0xbe, 0xef]), 0x92);
    }

    /// Test the `validate_crc` function.
    #[test]
    fn validate_crc() {
        let dev = hal::I2cMock::new();
        let sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);

        // Not enough data
        sgp.validate_crc(&[]).unwrap();
        sgp.validate_crc(&[0xbe]).unwrap();
        sgp.validate_crc(&[0xbe, 0xef]).unwrap();

        // Valid CRC
        sgp.validate_crc(&[0xbe, 0xef, 0x92]).unwrap();

        // Invalid CRC
        match sgp.validate_crc(&[0xbe, 0xef, 0x91]) {
            Err(Error::Crc) => {},
            Err(_) => panic!("Invalid error: Must be Crc"),
            Ok(_) => panic!("CRC check did not fail"),
        }

        // Valid CRC (8 bytes)
        sgp.validate_crc(&[0xbe, 0xef, 0x92, 0xbe, 0xef, 0x92, 0x00, 0x00]).unwrap();

        // Invalid CRC (8 bytes)
        match sgp.validate_crc(&[0xbe, 0xef, 0x91, 0xbe, 0xef, 0xff, 0x00, 0x00]) {
            Err(Error::Crc) => {},
            Err(_) => panic!("Invalid error: Must be Crc"),
            Ok(_) => panic!("CRC check did not fail"),
        }
    }

    /// Test the `read_with_crc` function.
    #[test]
    fn read_with_crc() {
        let mut buf = [0; 3];

        // Valid CRC
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0xbe, 0xef, 0x92]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        sgp.read_with_crc(&mut buf).unwrap();
        assert_eq!(buf, [0xbe, 0xef, 0x92]);

        // Invalid CRC
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0xbe, 0xef, 0x00]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        match sgp.read_with_crc(&mut buf) {
            Err(Error::Crc) => {},
            Err(_) => panic!("Invalid error: Must be Crc"),
            Ok(_) => panic!("CRC check did not fail"),
        }
        assert_eq!(buf, [0xbe, 0xef, 0x00]); // Buf was changed
    }

    /// Test the `serial` function
    #[test]
    fn serial() {
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0, 0, 129, 0, 100, 254, 204, 130, 135]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        let serial = sgp.serial().unwrap();
        assert_eq!(serial, [0, 0, 0, 100, 204, 130]);
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_ok() {
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0xD4, 0x00, 0xC6]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        assert!(sgp.selftest().unwrap());
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_fail() {
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0x12, 0x34, 0x37]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        assert!(!sgp.selftest().unwrap());
    }

    /// Test the `measure` function: Require initialization
    #[test]
    fn measure_initialization_required() {
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        match sgp.measure() {
            Err(Error::NotInitialized) => {},
            Ok(_) => panic!("Error::NotInitialized not returned"),
            Err(_) => panic!("Wrong error returned"),
        }
    }

    /// Test the `measure` function: Calculation of return values
    #[test]
    fn measure_success() {
        let mut dev = hal::I2cMock::new();
        dev.set_read_data(&[0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]);
        let mut sgp = Sgp30::new(dev, 0x58, hal::DelayMockNoop);
        sgp.init().unwrap();
        let (co2eq, tvoc) = sgp.measure().unwrap();
        assert_eq!(co2eq, 4_660);
        assert_eq!(tvoc, 54_274);
    }
}
