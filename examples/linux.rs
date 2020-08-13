use embedded_hal::blocking::delay::DelayMs;
use linux_embedded_hal::{I2cdev, Delay};
use sgp30::Sgp30;


fn measure_loop(sgp: &mut Sgp30<I2cdev, Delay>) -> ! {
    let mut i = 0;
    loop {
        if i != 0 {
            Delay.delay_ms(1000u16 - 12 - 25);
        }
        if i % 10 == 0 {
            let baseline = sgp.get_baseline().unwrap();
            println!("Baseline: {} / {}", baseline.co2eq, baseline.tvoc);
        }
        let measurements = sgp.measure().unwrap();
        let signals = sgp.measure_raw_signals().unwrap();
        println!("{}: CO₂eq = {} ppm, TVOC = {} ppb, H2 sig = {}, Ethanol sig = {}",
                 i + 1, measurements.co2eq_ppm, measurements.tvoc_ppb, signals.h2, signals.ethanol);
        i += 1;
    }
}

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = 0x58;
    let mut sgp = Sgp30::new(dev, address, Delay);

    println!("Starting SGP30 tests.");
    println!();
    println!("Serial: {:?}", sgp.serial().unwrap());
    println!("Feature set: {:?}", sgp.get_feature_set().unwrap());
    println!("Self-Test: {}", if sgp.selftest().unwrap() { "Pass" } else { "Fail" });
    println!();
    println!("Initializing...");
    sgp.init().unwrap();
    println!("Starting measurement loop, press Ctrl+C to abort...\n");
    measure_loop(&mut sgp);
}
