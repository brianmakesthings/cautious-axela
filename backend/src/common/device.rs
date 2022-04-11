pub mod door;
pub mod keypad;
pub mod nfc;
pub mod terminal;

use crate::message::{Receive, Send};
use std::{
    marker,
    thread::{self, JoinHandle},
    time::Duration,
};

pub fn launch_device<T, U>(device: impl Device<T, U>) -> JoinHandle<()> {
    thread::spawn(move || {
        device.run();
    })
}

pub struct Shutdown(pub bool);

pub trait Device<T, U>: Receive<T> + Send<U> + Sized + marker::Send + 'static {
    fn handle_command(&mut self, request: T) -> Shutdown;
    fn get_sleep_duration(&self) -> Option<Duration>;
    fn step(&mut self);
    fn run(mut self) {
        loop {
            let received_message = Self::receive(&mut self);
            if let Ok(msg) = received_message {
                let shutdown = self.handle_command(msg);
                if shutdown.0 {
                    break;
                };
            }
            self.step();
            match self.get_sleep_duration() {
                Some(duration) => std::thread::sleep(duration),
                _ => (),
            }
        }
    }
}
