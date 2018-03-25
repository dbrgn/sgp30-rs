//! SGP30

#![deny(unsafe_code)]
#![no_std]

extern crate embedded_hal as hal;

use hal::blocking::delay::DelayUs;
use hal::blocking::i2c::{Read, Write, WriteRead};


const CRC8_POLYNOMIAL: u8 = 0x31;


/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2c(E),
    /// CRC checksum validation failed
    Crc,
}


#[derive(Debug)]
pub enum Command {
    GetSerial,
}

impl Command {
    fn as_bytes(&self) -> [u8; 2] {
        match *self {
            Command::GetSerial => [0x36, 0x82],
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
}

impl<I2C, D, E> Sgp30<I2C, D>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayUs<u16>,
{
    pub fn new(i2c: I2C, address: u8, delay: D) -> Self {
        Sgp30 {
            i2c,
            address,
            delay,
        }
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

    /// Return the serial number of the SGP30.
    pub fn serial(&mut self) -> Result<u64, Error<E>> {
        // Request serial number
        let command = Command::GetSerial.as_bytes();
        self.i2c
            .write(self.address, &command)
            .map_err(Error::I2c)?;

        // Recommended wait time according to datasheet (6.5)
        self.delay.delay_us(500);

        // Read serial number
        let mut buf = [0; 9];
        self.read_with_crc(&mut buf)?;

        panic!("buf is {:?}", buf);

        Ok(0)
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

    /// Test the validate_crc function.
    #[test]
    fn validate_crc() {
        let dev = hal::I2cMock::new();
        let address = 0x58;
        let mut sgp = Sgp30::new(dev, address, hal::DelayMockNoop);

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
}
