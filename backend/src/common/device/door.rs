use super::{Device, Shutdown};
use crate::dispatch::Dispatcher;
use crate::message;
use crate::message::{Receive, Send, TcpSender, ThreadReceiver};
use crate::request::{Error, Get, GetRequest, Set, SetRequest};
use crate::requests_and_responses::{Requests, Responses, ThreadRequest};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use sysfs_gpio::Pin;

pub const PIN_NUMBER: u64 = 48;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum DoorState {
    Lock,
    Unlock,
}

impl DoorState {
    fn state_to_pin_value(&self) -> u8 {
        match &self {
            DoorState::Lock => 0,
            DoorState::Unlock => 1,
        }
    }
}

pub struct DoorDevice {
    sender: TcpSender<Responses>,
    receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
    door: Door,
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
            Requests::DoorGetState(x) => self
                .sender
                .send(Responses::DoorGetState(x.get_response(&self.door))),
            Requests::DoorSetState(x) => self
                .sender
                .send(Responses::DoorSetState(x.get_response(&mut self.door))),
            _ => panic!("Door device received invalid request"),
        }
        Shutdown(false)
    }
    fn get_sleep_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(500))
    }
    fn step(&mut self) {
        if self.door.state == DoorState::Unlock && self.door.last_unlocked.elapsed().as_secs() > 3 {
            match self.door.set(&DoorState::Lock) {
                Ok(()) => (),
                err => panic!("Unable to lock door after timeout: {:?}", err),
            }
        }
    }
}

impl DoorDevice {
    pub fn new(
        sender: TcpSender<Responses>,
        receiver: ThreadReceiver<ThreadRequest, Dispatcher>,
        door: Door,
    ) -> DoorDevice {
        return DoorDevice {
            sender,
            receiver,
            door,
        };
    }
}

#[derive(Clone)]
pub struct Door {
    state: DoorState,
    pin: Pin,
    last_unlocked: Instant,
}

impl Door {
    pub fn new(state: DoorState, pin: Pin, last_unlocked: Instant) -> Door {
        // let curTime = Instant::now();
        Door {
            state,
            pin,
            last_unlocked,
        }
    }
}

impl Set<Door, DoorState> for Door {
    fn set(&mut self, target: &DoorState) -> Result<(), Error> {
        println!("Setting door state to {:?}", target);
        let pin_value = target.state_to_pin_value();
        if let Err(_) = self.pin.set_value(pin_value) {
            return Err(Error("Could not set pin value".to_string()));
        }
        self.state = *target;
        if *target == DoorState::Unlock {
            self.last_unlocked = Instant::now();
        }
        Ok(())
    }
}

impl Get<Door, DoorState> for Door {
    fn get(&self) -> Result<DoorState, Error> {
        Ok(self.state)
    }
}
