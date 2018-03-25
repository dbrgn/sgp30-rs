extern crate linux_embedded_hal;
extern crate sgp30;

use linux_embedded_hal::{I2cdev, Delay};
use sgp30::Sgp30;


fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = 0x58;
    let mut sgp = Sgp30::new(dev, address, Delay);
}
