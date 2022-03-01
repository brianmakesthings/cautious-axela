use crate::device::{Terminal, Text};
use crate::request::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::io::Write;
use std::net::{SocketAddrV4, TcpStream};
use std::sync::mpsc;

pub trait Receive<T> {
    fn receive(self) -> Result<T, Error>
    where
        T: 'static + Send + DeserializeOwned;
}

pub struct Channel<T>(pub mpsc::Receiver<T>);

impl<T> Receive<T> for Channel<T> {
    fn receive(self) -> Result<T, Error>
    where
        T: 'static + Send + DeserializeOwned,
    {
        let result = self.0.try_recv();
        match result {
            Ok(x) => Ok(x),
            Err(mpsc::TryRecvError::Empty) => Err(Error("Empty".to_string())),
            _ => panic!("Thread disconnected"),
        }
    }
}

pub trait Route<T> {
    fn send(self, target: T) -> Result<(), Error>
    where
        T: Serialize + Send + 'static;
}

pub struct TCPRoute(SocketAddrV4);

impl<T> Route<T> for TCPRoute {
    fn send(self, target: T) -> Result<(), Error>
    where
        T: Serialize + Send + 'static,
    {
        let stream = TcpStream::connect(self.0);
        if let Err(_) = stream {
            return Err(Error("Could not connect to TCP stream".to_string()));
        }
        let mut stream = stream.unwrap();
        tokio::spawn(async move {
            let json = match serde_json::to_string(&target) {
                Ok(x) => x,
                Err(error) => panic!("Cannot convert into Json {0}", error),
            };
            match stream.write_all(json.as_bytes()) {
                Ok(_) => (),
                Err(_) => panic!("Cannot send Json over TCP"),
            };
        });
        Ok(())
    }
}

pub struct Message<T, R>(T, R);

pub enum Requests<T> {
    TerminalGetText(Message<T, BasicGetRequest<Terminal, Text>>),
    TerminalSetText(Message<T, BasicSetRequest<Terminal, Text>>),
}

pub enum Responses<T> {
    TerminalGetText(Message<T, BasicGetResponse<Terminal, Text>>),
    TerminalSetText(Message<T, BasicSetResponse<Terminal, Text>>),
}
