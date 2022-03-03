use crate::device::{Terminal, Text};
use crate::request::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::sync::mpsc;

pub struct ThreadSender<Type, To>(pub mpsc::Sender<Type>, pub PhantomData<To>);

impl<Type, To> Clone for ThreadSender<Type, To> {
    fn clone(&self) -> ThreadSender<Type, To> {
        ThreadSender(self.0.clone(), PhantomData)
    }
}

pub struct ThreadReceiver<Type, From>(pub mpsc::Receiver<Type>, pub PhantomData<From>);

#[derive(Serialize, Deserialize)]
pub enum Requests {
    TerminalGetText(BasicGetRequest<Terminal, Text>),
    TerminalSetText(BasicSetRequest<Terminal, Text>),
}

#[derive(Serialize, Deserialize)]
pub enum Responses {
    TerminalGetText(BasicGetResponse<Terminal, Text>),
    TerminalSetText(BasicSetResponse<Terminal, Text>),
}

pub struct ThreadRequest(pub Requests, pub TcpStream);

pub fn read_from_stream<T>(stream: &mut TcpStream) -> T
where
    T: DeserializeOwned,
{
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf).unwrap();
    let len: u64 = u64::from_le_bytes(len_buf);
    let mut buf = vec![0u8; usize::try_from(len).unwrap()];
    stream.read_exact(&mut buf[..]).unwrap();
    let json = String::from_utf8(buf).unwrap();
    let return_value: T = serde_json::from_str(&json).unwrap();
    return_value
}

pub fn write_to_stream<T>(stream: &mut TcpStream, target: T)
where
    T: Serialize,
{
    let json = serde_json::to_string(&target).unwrap();
    let bytes = json.as_bytes();
    stream
        .write_all(&(bytes.len() as u64).to_le_bytes())
        .unwrap();
    stream.write_all(bytes).unwrap();
    stream.flush().unwrap();
}
