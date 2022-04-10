use crate::device::door::Door;
use crate::device::keypad::KeyPad;
use crate::device::terminal::Terminal;
use crate::device::nfc::NFCdev;
use crate::message::{read_from_stream, ThreadSender};
use crate::requests_and_responses::{Requests, ThreadRequest};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone)]
pub struct Dispatcher {
    terminal_channel: ThreadSender<ThreadRequest, Terminal>,
    nfc_channel: ThreadSender<ThreadRequest, NFCdev>,
    door_channel: ThreadSender<ThreadRequest, Door>,
    keypad_channel: ThreadSender<ThreadRequest, KeyPad>,
}

impl Dispatcher {
    pub fn dispatch(&self, request: Requests, stream: TcpStream) {
        match request {
            Requests::TerminalGetText(_) => self
                .terminal_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::TerminalSetText(_) => self
                .terminal_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::NFCGetID(_) => self
                .nfc_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),    
            Requests::NFCSetID(_) => self
                .nfc_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::DoorGetState(_) => self
                .door_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::DoorSetState(_) => self
                .door_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::KeyPadGetCode(_) => self
                .keypad_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
            Requests::KeyPadSetCode(_) => self
                .keypad_channel
                .0
                .send(ThreadRequest(request, stream))
                .unwrap(),
        }
    }
    pub fn new(
        terminal_channel: ThreadSender<ThreadRequest, Terminal>,
        door_channel: ThreadSender<ThreadRequest, Door>,
        keypad_channel: ThreadSender<ThreadRequest, KeyPad>,
        nfc_channel: ThreadSender<ThreadRequest, NFCdev>,
    ) -> Dispatcher {
        Dispatcher {
            terminal_channel,
            door_channel,
            keypad_channel,
            nfc_channel,
        }
    }
}

pub fn start_server(dispatcher: Dispatcher, listener: TcpListener) {
    for stream in listener.incoming() {
        if let Err(_) = stream {
            continue;
        }
        let mut stream = stream.unwrap();
        {
            let dispatcher = dispatcher.clone();
            thread::spawn(move || {
                let request = read_from_stream(&mut stream).unwrap();
                dispatcher.dispatch(request, stream);
            });
        }
    }
}
