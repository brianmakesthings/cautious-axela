pub mod solenoid;

use gpio::{GpioIn, GpioOut};

const GPIO_PIN = 71;
let mut curState = false;

fn set_state(state: Bool) {
    let mut gpio = gpio::sysfs::SysFsGpioOuput::open(GPIO_PIN).unwrap()
    gpio.set_value(state).expect(format!("Could not set GPIO {}", GPIO_PIN));
    curState = state;
}

fn get_state() -> Bool {
    return curState;
}