use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::filters::ws::Message;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

pub struct Client {
    pub id: String,
    pub ws: mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
    pub rtc: Option<mpsc::Sender<()>>,
    pub video_track: Option<Arc<TrackLocalStaticRTP>>,
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub async fn broadcast(clients: Clients, msg: Message) {
    for client in clients.lock().await.values() {
        client.ws.send(Ok(msg.clone())).unwrap();
    }
}
