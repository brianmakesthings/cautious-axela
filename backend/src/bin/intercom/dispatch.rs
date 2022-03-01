use common::message::*;
use common::request::*;
use std::net::{SocketAddrV4, TcpStream};

struct Dispatcher {
    stream: TcpStream,
}

impl Dispatcher {
    fn dispatch_request(&self, request: Requests<TCPRoute>) {}
    fn new(address: &SocketAddrV4) -> Dispatcher {
        let stream = TcpStream::connect(address);
        if let Err(_) = stream {
            panic!("Dispatcher could not connect to backend");
        }
        let stream = stream.unwrap();
        Dispatcher { stream }
    }
}
