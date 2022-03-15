use crate::web_ws::{broadcast, Clients};
use std::net;
use std::thread;
use tokio::runtime::Runtime;
use warp::filters::ws::Message;

fn recv(socket: &net::UdpSocket, mut buffer: &mut [u8]) -> usize {
    let (n_bytes, _src_addr) = socket.recv_from(&mut buffer).expect("no data received");

    n_bytes
}

fn init(host: &str) -> net::UdpSocket {
    let socket = net::UdpSocket::bind(host).expect("failed to bind host socket");
    println!("binded udp {}", host);
    socket
}

// (temp) use this to receive sparse data from intercom
pub fn mainloop(clients: Clients) -> std::thread::JoinHandle<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(100);

    let socket = init("0.0.0.0:8001");

    thread::spawn(move || loop {
        let _len = recv(&socket, &mut buf);
        let fut = broadcast(clients.clone(), Message::text("{\"message\": \"test\"}"));
        Runtime::new().unwrap().block_on(fut);
    })
}
