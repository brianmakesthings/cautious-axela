/// Defines most of the traits and structures to communicate over threads/network.
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::sync::mpsc;

#[derive(Debug)]
pub enum Error {
    NotReady,
}

pub trait Send<Type> {
    fn send(&mut self, target: Type);
}

pub trait Receive<Type> {
    fn receive(&mut self) -> Result<Type, Error>;
}

pub struct ThreadSender<Type, To>(pub mpsc::Sender<Type>, pub PhantomData<To>);

impl<Type, To> Clone for ThreadSender<Type, To> {
    fn clone(&self) -> ThreadSender<Type, To> {
        ThreadSender(self.0.clone(), PhantomData)
    }
}

impl<Type, To> Send<Type> for ThreadSender<Type, To> {
    fn send(&mut self, target: Type) {
        self.0.send(target).unwrap();
    }
}

pub struct ThreadReceiver<Type, From>(pub mpsc::Receiver<Type>, pub PhantomData<From>);

impl<Type, From> Receive<Type> for ThreadReceiver<Type, From> {
    fn receive(&mut self) -> Result<Type, Error> {
        match self.0.try_recv() {
            Ok(x) => Ok(x),
            Err(mpsc::TryRecvError::Empty) => Err(Error::NotReady),
            _ => panic!("Cannot receive data."),
        }
    }
}

pub struct TcpSender<Type>(pub Option<TcpStream>, pub PhantomData<Type>);

impl<Type> Send<Type> for TcpSender<Type>
where
    Type: Serialize,
{
    fn send(&mut self, target: Type) {
        if let Some(ref mut stream) = self.0 {
            write_to_stream(stream, &target);
        }
    }
}

impl<Type> TcpSender<Type> {
    pub fn set_stream(&mut self, stream: TcpStream) {
        self.0 = Some(stream);
    }
}

pub struct TcpReceiver<Type>(pub TcpStream, pub PhantomData<Type>);

impl<Type> Receive<Type> for TcpReceiver<Type>
where
    Type: DeserializeOwned,
{
    fn receive(&mut self) -> Result<Type, Error> {
        read_from_stream(&mut self.0)
    }
}

pub fn read_from_stream<T>(stream: &mut impl Read) -> Result<T, Error>
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
    Ok(return_value)
}

pub fn write_to_stream(stream: &mut impl Write, target: &impl Serialize) {
    let json = serde_json::to_string(&target).unwrap();
    let bytes = json.as_bytes();
    let length = bytes.len() as u64;
    stream.write_all(&length.to_le_bytes()).unwrap();
    stream.write_all(bytes).unwrap();
    stream.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct ShortTestData(String);

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct LongTestData(Vec<i32>);

    #[test]
    fn test_thread_io() {
        let (tx, rx) = mpsc::channel();
        let mut sender = ThreadSender::<ShortTestData, ShortTestData>(tx, PhantomData);
        let mut receiver = ThreadReceiver::<ShortTestData, ShortTestData>(rx, PhantomData);
        let data = ShortTestData("Hello there".to_string());
        sender.send(data.clone());
        let result = receiver.receive().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_tcp_io() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(x) => x,
            _ => return,
        };
        let send_stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let receive_stream = listener.accept().unwrap();
        let mut sender = TcpSender::<ShortTestData>(Some(send_stream), PhantomData);
        let mut receiver = TcpReceiver::<ShortTestData>(receive_stream.0, PhantomData);
        let data = ShortTestData("Hello there".to_string());
        sender.send(data.clone());
        let result = receiver.receive().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_short_io_stream() {
        let mut buf = [0u8; 256];
        let data = ShortTestData("Hello There".to_string());
        write_to_stream(&mut buf.as_mut_slice(), &data);
        let result: ShortTestData = read_from_stream(&mut buf.as_slice()).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_long_io_stream() {
        let mut buf = [0u8; 65536];
        let vec = vec![63i32; 8192];
        let data = LongTestData(vec);
        write_to_stream(&mut buf.as_mut_slice(), &data);
        let result: LongTestData = read_from_stream(&mut buf.as_slice()).unwrap();
        assert_eq!(result, data);
    }
}
