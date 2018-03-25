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
    D: DelayUs<u8>,
{
    pub fn new(i2c: I2C, address: u8, delay: D) -> Self {
        Sgp30 {
            i2c,
            address,
            delay,
        }
    }

    pub fn serial(&mut self) -> Result<u64, Error<E>> {
        // Request serial number
        let command = [0x36, 0x82]; // TODO
        self.i2c
            .write(self.address, &command)
            .map_err(Error::I2c)?;

        // Recommended wait time according to datasheet (6.5)
        self.delay.delay_us(500);

        // Read serial number
        let mut buf = [0; 9];
        self.i2c
            .read(self.address, &mut buf)
            .map_err(Error::I2c)?;

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
    use super::*;

    /// Test the crc8 function against the test value provided in the
    /// datasheet (section 6.6).
    #[test]
    fn crc8_test_value() {
        assert_eq!(crc8(&[0xbe, 0xef]), 0x92);
    }
}
