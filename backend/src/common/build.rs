use crate::device::door;
use crate::device::terminal;
use crate::dispatch;
use crate::message;
use crate::requests_and_responses::ThreadRequest;
use std::marker::PhantomData;
use std::sync::mpsc;
use std::time::Instant;
use sysfs_gpio::{Direction, Pin};

pub trait Build {
    type Result;
    type Input;
    fn build(input: Self::Input) -> Self::Result;
}

impl Build for terminal::TerminalDevice {
    type Input = ();
    type Result = (
        message::ThreadSender<ThreadRequest, terminal::Terminal>,
        terminal::TerminalDevice,
    );
    fn build(_: ()) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver, PhantomData);
        let terminal = terminal::Terminal();
        let tcp_sender = message::TcpSender(None, PhantomData);
        let terminal_device = terminal::TerminalDevice::new(tcp_sender, thread_receiver, terminal);
        let terminal_channel = message::ThreadSender(sender, PhantomData);
        (terminal_channel, terminal_device)
    }
}

impl Build for dispatch::Dispatcher {
    type Result = dispatch::Dispatcher;
    type Input = (
        message::ThreadSender<ThreadRequest, terminal::Terminal>,
        message::ThreadSender<ThreadRequest, door::Door>,
    );
    fn build(input: Self::Input) -> Self::Result {
        dispatch::Dispatcher::new(input.0, input.1)
    }
}

impl Build for door::DoorDevice {
    type Input = ();
    type Result = (
        message::ThreadSender<ThreadRequest, door::Door>,
        door::DoorDevice,
    );
    fn build(_: ()) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver, PhantomData);
        let pin = Pin::new(door::PIN_NUMBER);
        if !pin.is_exported() {
            match pin.export() {
                Ok(()) => (),
                Err(error) => panic!("Got error when exported GPIO pin: {}", error),
            };
        }
        match pin.set_direction(Direction::Out) {
            Ok(()) => (),
            Err(error) => panic!("Unable to set door GPIO direction: {}", error),
        };
        let door = door::Door::new(door::DoorState::Lock, pin, Instant::now());
        let tcp_sender = message::TcpSender(None, PhantomData);
        let door_device = door::DoorDevice::new(tcp_sender, thread_receiver, door);
        let door_channel = message::ThreadSender(sender, PhantomData);
        (door_channel, door_device)
    }
}
