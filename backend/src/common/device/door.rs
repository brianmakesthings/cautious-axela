
use super::{Device, Shutdown};
use crate::dispatch::Dispatcher;
use crate::message;
use crate::message::{Receive, Send, TcpSender, ThreadReceiver};
use crate::request::{Error, Get, GetRequest, Set, SetRequest};
use crate::requests_and_responses::{Requests, Responses, ThreadRequest};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

use sysfs_gpio::{Pin, Direction};

const pin:u64 = 48;

pub enum DoorState {
    Lock,
    Unlock,
}

pub struct DoorDevice {
    sender: TcpSender<Responses>,
    receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
    door: Door,
    pin: Pin,
    state: DoorState,
}

impl Send<Responses> for DoorDevice {
    fn send(&mut self, target: Responses) {
        self.sender.send(target);
    }
}

impl Receive<ThreadRequest> for DoorDevice {
    fn receive(&mut self) -> Result<ThreadRequest, message::Error> {
        self.receiver.receive()
    }
}

impl Device<ThreadRequest, Responses> for DoorDevice {
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

impl DoorDevice {
    pub fn new(
        sender: TcpSender<Responses>,
        receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
        door: Door,
    ) -> DoorDevice {
        let doorPin: Pin = Pin::new(pin);
        match doorPin.export() {
            Ok(()) => Ok(()),
            Err(error) => println!("Got error when exported GPIO pin: {}", error),
        };
        match doorPin.set_direction(Direction::Out) {
            Ok(()) => Ok(()),
            Err(error) => println!("Unable to set door GPIO direction: {}", error),
        };
        // assume for now if error, then gpio pin already exported
        return DoorDevice {
            sender,
            receiver,
            door,
            doorPin,
            DoorState::Lock
        };
    }
}

#[derive(Clone)]
pub struct Door();
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Text(pub String);

impl Set<Door, DoorState> for Door {
    fn set(&mut self, target: DoorState) -> Result<(), Error> {
        println!("Setting door state to {}", target);
        self.pin.set_value(target)?;
        self.state = target;
        Ok(())
    }
}

impl Get<Door, DoorState> for Door {
    fn get(&self) -> Result<DoorState, Error> {
        Ok(self.state)
    }
}
