use crate::web_ws::Clients;
use std::thread;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

async fn init(host: &str) -> UdpSocket {
    let socket = UdpSocket::bind(host)
        .await
        .expect("failed to bind host socket");
    println!("RTP listening on {}", host);
    socket
}

async fn rtp_loop(clients: &Clients) {
    let socket = init("0.0.0.0:8002").await;

    let mut inbound_rtp_packet = vec![0u8; 1600]; // UDP MTU
    while let Ok((n, _)) = socket.recv_from(&mut inbound_rtp_packet).await {
        /*if let Err(err) = video_track.write(&inbound_rtp_packet[..n]).await {
            if Error::ErrClosedPipe == err {
                // The peerConnection has been closed.
            } else {
                println!("video_track write err: {}", err);
            }
            let _ = done_tx3.try_send(());
            return;
        }*/
    }
}

pub fn mainloop(clients: Clients) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let fut = rtp_loop(&clients);
        Runtime::new().unwrap().block_on(fut);
    })
}
