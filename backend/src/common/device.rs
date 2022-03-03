use crate::{
    message::{write_to_stream, Requests, ThreadReceiver, ThreadRequest},
    request::{Error, Get, GetRequest, Set, SetRequest},
};
use serde::{Deserialize, Serialize};
use std::io;

pub struct Terminal();
#[derive(Serialize, Deserialize, Debug)]
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

impl Terminal {
    fn process_command(&mut self, mut request: ThreadRequest) {
        match request.0 {
            Requests::TerminalGetText(x) => write_to_stream(&mut request.1, x.get_response(self)),
            Requests::TerminalSetText(x) => write_to_stream(&mut request.1, x.get_response(self)),
        }
    }
}

pub fn driver(receiver: ThreadReceiver<ThreadRequest, Terminal>, mut terminal: Terminal) {
    let rx = receiver.0;
    for request in rx.iter() {
        terminal.process_command(request);
    }
}
