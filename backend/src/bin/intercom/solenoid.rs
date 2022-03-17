pub mod solenoid;
use gpio::{GpioIn, GpioOut};

const GPIO_PIN = 48;
fn set_state(state: Bool) {
    // true is locked, false is unlocked
    let mut gpio = gpio::sysfs::SysFsGpioOuput::open(GPIO_PIN).unwrap()
    gpio.set_value(!state).expect(format!("Could not set GPIO {}", GPIO_PIN));
    // println!("GPIO PIN IS {}", GPIO_PIN);
}