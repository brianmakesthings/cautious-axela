use super::{Device, Shutdown};
use crate::dispatch::Dispatcher;
use crate::message;
use crate::message::{Receive, Send, TcpSender, ThreadReceiver};
use crate::request::{Error, Get, GetRequest, Set, SetRequest};
use crate::requests_and_responses::{Requests, Responses, ThreadRequest};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

pub struct TerminalDevice {
    sender: TcpSender<Responses>,
    receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
    terminal: Terminal,
}

impl Send<Responses> for TerminalDevice {
    fn send(&mut self, target: Responses) {
        self.sender.send(target);
    }
}

impl Receive<ThreadRequest> for TerminalDevice {
    fn receive(&mut self) -> Result<ThreadRequest, message::Error> {
        self.receiver.receive()
    }
}

impl Device<ThreadRequest, Responses> for TerminalDevice {
    fn handle_command(&mut self, request: ThreadRequest) -> Shutdown {
        let ThreadRequest(request, stream) = request;
        self.sender.set_stream(stream);
        match request {
            Requests::TerminalGetText(x) => self
                .sender
                .send(Responses::TerminalGetText(x.get_response(&self.terminal))),

            Requests::TerminalSetText(x) => self.sender.send(Responses::TerminalSetText(
                x.get_response(&mut self.terminal),
            )),
        }
        Shutdown(false)
    }
    fn get_sleep_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(250))
    }
    fn step(&mut self) {}
}

impl TerminalDevice {
    pub fn new(
        sender: TcpSender<Responses>,
        receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
        terminal: Terminal,
    ) -> TerminalDevice {
        return TerminalDevice {
            sender,
            receiver,
            terminal,
        };
    }
}

#[derive(Clone)]
pub struct Terminal();
#[derive(Serialize, Deserialize, Debug, Clone)]
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
