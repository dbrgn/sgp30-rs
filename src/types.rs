use core::f32;

/// Absolute humidity in g/m³.
///
/// Internally this is represented as a 8.8bit fixed-point number.
///
/// To construct a `Humidity` instance, either use the lossless `new()`
/// constructor, or the lossy `From<f32>` implementation.
#[derive(Debug, PartialEq, Eq)]
pub struct Humidity {
	integer: u8, // 0-255
	fractional: u8, // 0/256-255/256
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
	pub fn new(integer: u8, fractional: u8) -> Self {
		Humidity { integer, fractional }
	}

	/// Convert this to the binary fixed-point representation expected by the
	/// SGP30 sensor.
    fn as_bytes(&self) -> [u8; 2] {
		[self.integer, self.fractional]
	}
}

impl From<f32> for Humidity {
	/// Create a new `Humidity` instance from a f32.
    ///
    /// The humidity will be clipped to the representable range
    /// (roughly 0-256). The fractional part will always be rounded down.
    ///
    /// The code will panic if `NaN` is passed in.
    fn from(val: f32) -> Self {
        if val.is_nan() {
            panic!("NaN cannot be converted to Humidity");
        }

        let integer = if val >= 255.0 {
            255
        } else if val <= 0.0 {
            0
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

        Humidity { integer, fractional }
    }
}

impl Into<f32> for Humidity {
    /// Convert a `Humidity` instance to a f32.
    fn into(self) -> f32 {
        self.integer as f32 + (self.fractional as f32 / 256.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humidity_as_bytes() {
        assert_eq!(Humidity::new(0x00, 0x01).as_bytes(), [0x00, 0x01]);
        assert_eq!(Humidity::new(0xFF, 0xFF).as_bytes(), [0xFF, 0xFF]);
        assert_eq!(Humidity::new(0x10, 0x80).as_bytes(), [0x10, 0x80]);
    }

    #[test]
    fn humidity_from_f32() {
        assert_eq!(Humidity::from(0.00390625f32), Humidity::new(0x00, 0x01));
        assert_eq!(Humidity::from(255.99609375f32), Humidity::new(0xFF, 0xFF));
        assert_eq!(Humidity::from(16.5f32), Humidity::new(0x10, 0x80));
        assert_eq!(Humidity::from(16.999999f32), Humidity::new(0x10, 0xFF));
        assert_eq!(Humidity::from(-3.0f32), Humidity::new(0x00, 0x00));
        assert_eq!(Humidity::from(0.0f32), Humidity::new(0x00, 0x00));
        assert_eq!(Humidity::from(-0.0f32), Humidity::new(0x00, 0x00));
    }

    #[test]
    #[should_panic]
    fn humidity_from_f32_nan() {
        Humidity::from(f32::NAN);
    }

    #[test]
    fn humidity_into_f32() {
        let float: f32 = Humidity::new(0x00, 0x01).into();
        assert_eq!(float, 0.00390625f32);
        let float: f32 = Humidity::new(0xFF, 0xFF).into();
        assert_eq!(float, 255.99609375);
        let float: f32 = Humidity::new(0x10, 0x80).into();
        assert_eq!(float, 16.5);
    }
}
