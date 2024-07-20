use super::{types::*, Command, Error, SELFTEST_SUCCESS};
use byteorder::{BigEndian, ByteOrder};
use embedded_hal_async::{delay::DelayNs, i2c::I2c};
use sensirion_i2c::i2c_async;

/// Async driver for the SGP30.
///
/// This type is identical to the [`Sgp30`](crate::Sgp30) type, but using the
/// [`embedded_hal_async`] versions of the [`I2c`] and [`DelayNs`] traits.
#[derive(Debug, Default)]
pub struct Sgp30Async<I2C, D> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The I²C device address.
    address: u8,
    /// The concrete Delay implementation.
    delay: D,
    /// Whether the air quality measurement was initialized.
    initialized: bool,
}

impl<I2C, D> Sgp30Async<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    /// Create a new instance of the SGP30 driver.
    pub fn new(i2c: I2C, address: u8, delay: D) -> Self {
        Self {
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
    async fn send_command(&mut self, command: Command) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &command.as_bytes())
            .await
            .map_err(Error::I2cWrite)
    }

    /// Write an I²C command and data to the sensor.
    ///
    /// The data slice must have a length of 2 or 4.
    ///
    /// CRC checksums will automatically be added to the data.
    async fn send_command_and_data(
        &mut self,
        command: Command,
        data: &[u8],
    ) -> Result<(), Error<I2C::Error>> {
        let mut buf = [0; 2 /* command */ + 6 /* max length of data + crc */];
        let payload = command.as_bytes_with_data(&mut buf, data);
        self.i2c
            .write(self.address, payload)
            .await
            .map_err(Error::I2cWrite)
    }

    /// Return the 48 bit serial number of the SGP30.
    pub async fn serial(&mut self) -> Result<[u8; 6], Error<I2C::Error>> {
        // Request serial number
        self.send_command(Command::GetSerial).await?;

        // Recommended wait time according to datasheet (6.5)
        self.delay.delay_us(500).await;

        // Read serial number
        let mut buf = [0; 9];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;

        Ok([buf[0], buf[1], buf[3], buf[4], buf[6], buf[7]])
    }

    /// Run an on-chip self-test. Return a boolean indicating whether the test succeeded.
    pub async fn selftest(&mut self) -> Result<bool, Error<I2C::Error>> {
        // Start self test
        self.send_command(Command::SelfTest).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(220).await;

        // Read result
        let mut buf = [0; 3];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;

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
    pub async fn init(&mut self) -> Result<(), Error<I2C::Error>> {
        if self.initialized {
            // Already initialized
            return Ok(());
        }
        self.force_init().await
    }

    /// Like [`init()`](Self::init), but without checking
    /// whether the sensor is already initialized.
    ///
    /// This might be necessary after a sensor soft or hard reset.
    pub async fn force_init(&mut self) -> Result<(), Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::InitAirQuality).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10).await;

        self.initialized = true;
        Ok(())
    }

    /// Get an air quality measurement.
    ///
    /// Before calling this method, the air quality measurements must have been
    /// initialized using the [`init()`](Self::init) method.
    /// Otherwise an [`Error::NotInitialized`] will be returned.
    ///
    /// Once the measurements have been initialized, the
    /// [`measure()`](Self::measure) method must be called
    /// in regular intervals of 1 s to ensure proper operation of the dynamic
    /// baseline compensation algorithm. It is the responsibility of the user
    /// of this driver to ensure that these periodic measurements are being
    /// done.
    ///
    /// For the first 15 s after initializing the air quality measurement, the
    /// sensor is in an initialization phase during which it returns fixed
    /// values of 400 ppm CO₂eq and 0 ppb TVOC. After 15 s (15 measurements)
    /// the values should start to change.
    pub async fn measure(&mut self) -> Result<Measurement, Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command to sensor
        self.send_command(Command::MeasureAirQuality).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(12).await;

        // Read result
        let mut buf = [0; 6];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;
        Ok(Measurement::from_bytes(&buf))
    }

    /// Return sensor raw signals.
    ///
    /// This command is intended for part verification and testing purposes. It
    /// returns the raw signals which are used as inputs for the on-chip
    /// calibration and baseline compensation algorithm. The command performs a
    /// measurement to which the sensor responds with the two signals for H2
    /// and Ethanol.
    pub async fn measure_raw_signals(&mut self) -> Result<RawSignals, Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command to sensor
        self.send_command(Command::MeasureRawSignals).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(25).await;

        // Read result
        let mut buf = [0; 6];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;
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
    /// algorithm can be restored by calling [`init()`](Self::init) followed by
    /// [`set_baseline()`](Self::set_baseline).
    pub async fn get_baseline(&mut self) -> Result<Baseline, Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::GetBaseline).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10).await;

        // Read result
        let mut buf = [0; 6];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;
        Ok(Baseline::from_bytes(&buf))
    }

    /// Set the baseline values for the baseline correction algorithm.
    ///
    /// Before calling this method, the air quality measurements must have been
    /// initialized using the [`init()`](Self::init) method.
    /// Otherwise an [`Error::NotInitialized`]  will be returned.
    ///
    /// The SGP30 provides the possibility to read and write the baseline
    /// values of the baseline correction algorithm. This feature is used to
    /// save the baseline in regular intervals on an external non-volatile
    /// memory and restore it after a new power-up or soft reset of the sensor.
    ///
    /// This function sets the baseline values for the two air quality
    /// signals.
    pub async fn set_baseline(&mut self, baseline: &Baseline) -> Result<(), Error<I2C::Error>> {
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
        self.send_command_and_data(Command::SetBaseline, &buf)
            .await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10).await;

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
    /// initialized using the [`init()`](Self::init) method.
    /// Otherwise an [`Error::NotInitialized`] will be returned.
    pub async fn set_humidity(
        &mut self,
        humidity: Option<&Humidity>,
    ) -> Result<(), Error<I2C::Error>> {
        if !self.initialized {
            // Measurements weren't initialized
            return Err(Error::NotInitialized);
        }

        // Send command and data to sensor
        let buf = match humidity {
            Some(humi) => humi.as_bytes(),
            None => [0, 0],
        };
        self.send_command_and_data(Command::SetHumidity, &buf)
            .await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(10).await;

        Ok(())
    }

    /// Get the feature set.
    ///
    /// The SGP30 features a versioning system for the available set of
    /// measurement commands and on-chip algorithms. This so called feature set
    /// version number can be read out with this method.
    pub async fn get_feature_set(&mut self) -> Result<FeatureSet, Error<I2C::Error>> {
        // Send command to sensor
        self.send_command(Command::GetFeatureSet).await?;

        // Max duration according to datasheet (Table 10)
        self.delay.delay_ms(2).await;

        // Read result
        let mut buf = [0; 3];
        i2c_async::read_words_with_crc(&mut self.i2c, self.address, &mut buf).await?;

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
    use futures_executor::block_on;

    /// Test the `serial` function
    #[test]
    fn serial() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::GetSerial.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0, 0, 129, 0, 100, 254, 204, 130, 135]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            let serial = sgp.serial().await.unwrap();
            assert_eq!(serial, [0, 0, 0, 100, 204, 130]);
            sgp.destroy().done();
        })
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_ok() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::SelfTest.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0xD4, 0x00, 0xC6]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            assert!(sgp.selftest().await.unwrap());
            sgp.destroy().done();
        })
    }

    /// Test the `selftest` function
    #[test]
    fn selftest_fail() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::SelfTest.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0x12, 0x34, 0x37]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            assert!(!sgp.selftest().await.unwrap());
            sgp.destroy().done();
        })
    }

    /// Test the `measure` function: Require initialization
    #[test]
    fn measure_initialization_required() {
        block_on(async {
            let mock = I2cMock::new(&[]);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            match sgp.measure().await {
                Err(Error::NotInitialized) => {}
                Ok(_) => panic!("Error::NotInitialized not returned"),
                Err(_) => panic!("Wrong error returned"),
            }
            sgp.destroy().done();
        })
    }

    /// Test the `measure` function: Calculation of return values
    #[test]
    fn measure_success() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
                Transaction::write(0x58, Command::MeasureAirQuality.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let measurements = sgp.measure().await.unwrap();
            assert_eq!(measurements.co2eq_ppm, 4_660);
            assert_eq!(measurements.tvoc_ppb, 54_274);
            sgp.destroy().done();
        })
    }

    /// Test the `get_baseline` function
    #[test]
    fn get_baseline() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
                Transaction::write(0x58, Command::GetBaseline.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0xD4, 0x02, 0xA4]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let baseline = sgp.get_baseline().await.unwrap();
            assert_eq!(baseline.co2eq, 4_660);
            assert_eq!(baseline.tvoc, 54_274);
            sgp.destroy().done();
        })
    }

    /// Test the `set_baseline` function
    #[test]
    fn set_baseline() {
        block_on(async {
            #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x1E,
                /* data + crc8: */ 0x56, 0x78, 0x7D, 0x12, 0x34, 0x37,
            ]),
        ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let baseline = Baseline {
                co2eq: 0x1234,
                tvoc: 0x5678,
            };
            sgp.set_baseline(&baseline).await.unwrap();
            sgp.destroy().done();
        })
    }

    /// Test the `set_humidity` function
    #[test]
    fn set_humidity() {
        block_on(async {
            #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x61,
                /* data + crc8: */ 0x0F, 0x80, 0x62,
            ]),
        ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let humidity = Humidity::from_f32(15.5).unwrap();
            sgp.set_humidity(Some(&humidity)).await.unwrap();
            sgp.destroy().done();
        })
    }

    /// Test the `set_humidity` function with a None value
    #[test]
    fn set_humidity_none() {
        block_on(async {
            #[rustfmt::skip]
        let expectations = [
            Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
            Transaction::write(0x58, vec![
                /* command: */ 0x20, 0x61,
                /* data + crc8: */ 0x00, 0x00, 0x81,
            ]),
        ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            sgp.set_humidity(None).await.unwrap();
            sgp.destroy().done();
        })
    }

    /// Test the `get_feature_set` function.
    #[test]
    fn get_feature_set() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
                Transaction::write(0x58, Command::GetFeatureSet.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0x00, 0x42, 0xDE]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let feature_set = sgp.get_feature_set().await.unwrap();
            assert_eq!(feature_set.product_type, ProductType::Sgp30);
            assert_eq!(feature_set.product_version, 0x42);
            sgp.destroy().done();
        })
    }

    /// Test the `measure_raw_signals` function.
    #[test]
    fn measure_raw_signals() {
        block_on(async {
            let expectations = [
                Transaction::write(0x58, Command::InitAirQuality.as_bytes()[..].into()),
                Transaction::write(0x58, Command::MeasureRawSignals.as_bytes()[..].into()),
                Transaction::read(0x58, vec![0x12, 0x34, 0x37, 0x56, 0x78, 0x7D]),
            ];
            let mock = I2cMock::new(&expectations);
            let mut sgp = Sgp30Async::new(mock, 0x58, NoopDelay);
            sgp.init().await.unwrap();
            let signals = sgp.measure_raw_signals().await.unwrap();
            assert_eq!(signals.h2, (0x12 << 8) + 0x34);
            assert_eq!(signals.ethanol, (0x56 << 8) + 0x78);
            sgp.destroy().done();
        })
    }
}
