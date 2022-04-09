use crate::device::door;
use crate::device::keypad;
use crate::device::terminal;
use crate::dispatch;
use crate::message;
use crate::message::ThreadSender;
use crate::requests_and_responses::InternalThreadRequest;
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
        let thread_receiver = message::ThreadReceiver(receiver);
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
        message::ThreadSender<ThreadRequest, keypad::KeyPad>,
    );
    fn build(input: Self::Input) -> Self::Result {
        dispatch::Dispatcher::new(input.0, input.1, input.2)
    }
}

impl Build for door::DoorDevice {
    type Input = ();
    type Result = (
        message::ThreadSender<ThreadRequest, door::Door>,
        message::ThreadSender<InternalThreadRequest, door::Door>,
        door::DoorDevice,
    );
    fn build(_: ()) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let (internal_sender, internal_receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver);
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
        let internal_door_receiver = message::ThreadReceiver(internal_receiver);
        let door_device = door::DoorDevice::new(tcp_sender, thread_receiver, door, internal_door_receiver);
        let door_channel = message::ThreadSender(sender, PhantomData);
        let internal_door_sender = message::ThreadSender(internal_sender, PhantomData);
        (door_channel, internal_door_sender, door_device)
    }
}

impl Build for keypad::KeyPadMatrix {
    type Input = ();
    type Result = keypad::KeyPadMatrix;
    fn build(_: ()) -> Self::Result {
        let rows: [Pin; 4] = keypad::KeyPadMatrix::ROW_PINS.map(|x| Pin::new(x));
        let cols: [Pin; 4] = keypad::KeyPadMatrix::COL_PINS.map(|x| Pin::new(x));
        rows.iter().chain(cols.iter()).for_each(|x| {
            if !x.is_exported() {
                match x.export() {
                    Ok(()) => (),
                    Err(error) => panic!("Got error when exported GPIO pin: {}", error),
                };
            }
        });
        keypad::KeyPadMatrix::new(rows, cols)
    }
}

impl Build for keypad::KeyPadDevice {
    type Input = ThreadSender<InternalThreadRequest, door::Door>;
    type Result = (
        message::ThreadSender<ThreadRequest, keypad::KeyPad>,
        keypad::KeyPadDevice,
    );
    fn build(keypad_to_door_sender: Self::Input) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver);
        let code = keypad::Code::new();
        let keypad_matrix = keypad::KeyPadMatrix::build(());
        let candidate_key = keypad::CandidateKey::new(keypad::CandidateKey::INITIAL_CAPACITY);
        let keypad = keypad::KeyPad::new(code, keypad_matrix, candidate_key, Instant::now());
        let tcp_sender = message::TcpSender(None, PhantomData);
        let keypad_channel = message::ThreadSender(sender, PhantomData);
        let keypad_device =
            keypad::KeyPadDevice::new(keypad_to_door_sender, tcp_sender, thread_receiver, keypad);
        (keypad_channel, keypad_device)
    }
}
