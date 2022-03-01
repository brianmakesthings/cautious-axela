use serde::{Deserialize, Serialize};
use std::io;
use std::sync::mpsc;

use crate::{
    message::Channel,
    request::{Error, Get, Set},
};

pub struct Terminal();
#[derive(Serialize, Deserialize)]
pub struct Text(pub String);

impl Set<Terminal, Text> for Terminal {
    fn set(&mut self, target: &Text) -> Result<(), Error> {
        println!("{}", target.0);
        Ok(())
    }
}

// Blocking
impl Get<Terminal, Text> for Terminal {
    fn get(&self) -> Result<Text, Error> {
        let stdin = io::stdin();
        let mut string = String::new();
        let result = stdin.read_line(&mut string);
        match result {
            Ok(0) => Ok(Text("".to_string())),
            Ok(_) => Ok(Text(string)),
            _ => Err(Error("Could not read from terminal".to_string())),
        }
    }
}
