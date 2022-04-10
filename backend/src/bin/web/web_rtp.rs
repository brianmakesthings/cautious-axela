use std::sync::Arc;
use std::thread;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocalWriter;
use webrtc::Error;

async fn init(host: &str) -> UdpSocket {
    let socket = UdpSocket::bind(host)
        .await
        .expect("failed to bind host socket");
    println!("RTP listening on {}", host);
    socket
}

async fn rtp_loop(track: Arc<TrackLocalStaticRTP>, port: u32) {
    let socket = init(format!("0.0.0.0:{port}").as_str()).await;

    let mut inbound_rtp_packet = vec![0u8; 1600]; // UDP MTU
    while let Ok((n, _)) = socket.recv_from(&mut inbound_rtp_packet).await {
        if let Err(err) = track.write(&inbound_rtp_packet[..n]).await {
            if Error::ErrClosedPipe == err {
                println!("peer connection closed");
            } else {
                println!("track write err: {}", err);
            }
        }
    }
}

pub fn mainloop(track: Arc<TrackLocalStaticRTP>, port: u32) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let fut = rtp_loop(track, port);
        Runtime::new().unwrap().block_on(fut);
    })
}
