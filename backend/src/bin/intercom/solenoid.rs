use std::fs::File;
use std::io::{Write, BufReader, BufRead, Error};

// const GPIO_PIN:u64 = 48;
pub fn set_state(state: u8) -> Result<(), Error> {
    // let exportPath = "/sys/class/gpio/export";

    // let mut exportFile = File::create(exportPath)?;
    // write!(exportFile, "48")?;

    let direction_path = "/sys/class/gpio/gpio48/direction";
    let mut direction_file = File::create(direction_path)?;
    write!(direction_file, "out")?;

    let value_path = "/sys/class/gpio/gpio48/value";
    let mut value_file = File::create(value_path)?;
    write!(value_file, "1")?;
    println!("Ok!");
    Ok(())
}