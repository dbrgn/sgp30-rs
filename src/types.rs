#[allow(unused_imports)] // Required for no_std
use num_traits::float::FloatCore;

/// A measurement result from the sensor.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Measurement {
    /// CO₂ equivalent (parts per million, ppm)
	pub co2eq_ppm: u16,
    /// Total Volatile Organic Compounds (parts per billion, ppb)
	pub tvoc_ppb: u16,
}

/// A raw signals result from the sensor.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawSignals {
    /// H2 signal
	pub h2: u16,
    /// Ethanol signal
	pub ethanol: u16,
}

/// The baseline values..
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Baseline {
    /// CO₂eq baseline
	pub co2eq: u16,
    /// TVOC baseline
	pub tvoc: u16,
}

/// Absolute humidity in g/m³.
///
/// Internally this is represented as a 8.8bit fixed-point number.
///
/// To construct a `Humidity` instance, either use the lossless `new()`
/// constructor, or the lossy `from_f32()` method.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Humidity {
	integer: u8, // 0-255
	fractional: u8, // 0/256-255/256
}

/// Errors that can occur when constructing a `Humidity` value.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum HumidityError {
    /// A zero value is not allowed in a `Humidity` struct since that will turn
    /// off the temperature compensation.
    ZeroValue,
    /// A value is outside the representable range.
    OutOfRange,
}

impl Humidity {
	/// Create a new `Humidity` instance.
    ///
    /// The humidity should be passed in as a 8.8bit fixed-point number.
    ///
    /// Examples:
    ///
    /// - The pair `(0x00, 0x01)` represents `1/256 g/m³` (0.00390625)
    /// - The pair `(0xFF, 0xFF)` represents `255 g/m³ + 255/256 g/m³` (255.99609375)
    /// - The pair `(0x10, 0x80)` represents `16 g/m³ + 128/256 g/m³` (16.5)
	pub fn new(integer: u8, fractional: u8) -> Result<Self, HumidityError> {
        if integer == 0 && fractional == 0 {
            return Err(HumidityError::ZeroValue);
        }
		Ok(Humidity { integer, fractional })
	}

	/// Create a new `Humidity` instance from a f32.
    ///
    /// When converting, the fractional part will always be rounded down.
    pub fn from_f32(val: f32) -> Result<Self, HumidityError> {
        if val.is_nan() {
            return Err(HumidityError::OutOfRange);
        }

        let integer = if val >= 256.0 {
            return Err(HumidityError::OutOfRange);
        } else if val < 0.0 {
            return Err(HumidityError::OutOfRange);
        } else {
            val.trunc() as u8
        };

        let fractional_f32 = val.fract() * 256.0f32;
        let fractional = if fractional_f32 > 255.0 {
            255
        } else if fractional_f32 < 0.0 {
            0
        } else {
            fractional_f32 as u8
        };

        Humidity::new(integer, fractional)
    }

	/// Convert this to the binary fixed-point representation expected by the
	/// SGP30 sensor.
    pub fn as_bytes(&self) -> [u8; 2] {
		[self.integer, self.fractional]
	}
}

impl Into<f32> for Humidity {
    /// Convert a `Humidity` instance to a f32.
    fn into(self) -> f32 {
        self.integer as f32 + (self.fractional as f32 / 256.0)
    }
}

/// The product types compatible with this driver.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ProductType {
    /// SGP30
    Sgp30,
    /// Unknown product type
    Unknown(u8),
}

impl ProductType {
    /// Parse the product type.
    pub fn parse(val: u8) -> Self {
        match val {
            0 => ProductType::Sgp30,
            _ => ProductType::Unknown(val),
        } 
    }
}

/// The feature set returned by the sensor.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureSet {
    /// The product type (see [`ProductType`](enum.ProductType.html))
    pub product_type: ProductType,
    /// The product version
    pub product_version: u8,
}

impl FeatureSet {
    /// Parse the two bytes returned by the device.
    pub fn parse(msb: u8, lsb: u8) -> Self {
        FeatureSet {
            product_type: ProductType::parse(msb >> 4),
            product_version: lsb,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f32;

    use super::*;

    #[test]
    fn humidity_as_bytes() {
        assert_eq!(Humidity::new(0x00, 0x01).unwrap().as_bytes(), [0x00, 0x01]);
        assert_eq!(Humidity::new(0xFF, 0xFF).unwrap().as_bytes(), [0xFF, 0xFF]);
        assert_eq!(Humidity::new(0x10, 0x80).unwrap().as_bytes(), [0x10, 0x80]);
    }

    #[test]
    fn humidity_from_f32_ok() {
        assert_eq!(Humidity::from_f32(0.00390625f32), Ok(Humidity::new(0x00, 0x01).unwrap()));
        assert_eq!(Humidity::from_f32(255.99609375f32), Ok(Humidity::new(0xFF, 0xFF).unwrap()));
        assert_eq!(Humidity::from_f32(16.5f32), Ok(Humidity::new(0x10, 0x80).unwrap()));
        assert_eq!(Humidity::from_f32(16.999999f32), Ok(Humidity::new(0x10, 0xFF).unwrap()));
    }

    #[test]
    fn humidity_from_f32_err() {
        assert_eq!(Humidity::from_f32(-3.0f32), Err(HumidityError::OutOfRange));
        assert_eq!(Humidity::from_f32(0.0f32), Err(HumidityError::ZeroValue));
        assert_eq!(Humidity::from_f32(-0.0f32), Err(HumidityError::ZeroValue));
        assert_eq!(Humidity::from_f32(f32::NAN), Err(HumidityError::OutOfRange));
    }

    #[test]
    fn humidity_into_f32() {
        let float: f32 = Humidity::new(0x00, 0x01).unwrap().into();
        assert_eq!(float, 0.00390625f32);
        let float: f32 = Humidity::new(0xFF, 0xFF).unwrap().into();
        assert_eq!(float, 255.99609375);
        let float: f32 = Humidity::new(0x10, 0x80).unwrap().into();
        assert_eq!(float, 16.5);
    }
}
