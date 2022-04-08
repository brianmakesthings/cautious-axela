use crate::device::terminal::Terminal;
use crate::device::nfc::NFC;
use crate::message::{read_from_stream, ThreadSender};
use crate::requests_and_responses::{Requests, ThreadRequest};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone)]
pub struct Dispatcher {
    terminal_channel: ThreadSender<ThreadRequest, Terminal>,
    nfc_channel: ThreadSender<ThreadRequest, NFC>,
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
        }
    }
    pub fn new(
        terminal_channel: ThreadSender<ThreadRequest, Terminal>,
        nfc_channel: ThreadSender<ThreadRequest, NFC>,
    ) -> Dispatcher {
        Dispatcher {
            terminal_channel,
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
