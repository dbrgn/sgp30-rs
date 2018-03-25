extern crate embedded_hal;
extern crate linux_embedded_hal;
extern crate sgp30;

use embedded_hal::blocking::delay::DelayMs;
use linux_embedded_hal::{I2cdev, Delay};
use sgp30::Sgp30;


fn measure_loop(count: u16, sgp: &mut Sgp30<I2cdev, Delay>) {
    for i in 0..count {
        if i != 0 {
            Delay.delay_ms(1000u16);
        }
        let (co2eq, tvoc) = sgp.measure().unwrap();
        println!("{:2}: CO₂eq = {} ppm, TVOC = {} ppb", i + 1, co2eq, tvoc);
    }
}

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = 0x58;
    let mut sgp = Sgp30::new(dev, address, Delay);

    println!("Starting SGP30 tests.");
    println!();
    println!("Serial: {:?}", sgp.serial().unwrap());
    println!("Self-Test: {}", if sgp.selftest().unwrap() { "Pass" } else { "Fail" });
    println!();
    println!("Initializing...");
    sgp.init().unwrap();
    println!("Starting measurement loop for 60s...\n");
    measure_loop(60, &mut sgp);
}
