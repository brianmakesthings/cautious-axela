use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::filters::ws::Message;

pub struct Client {
    pub id: String,
    pub ws: mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
    pub rtc: Option<mpsc::Sender<()>>,
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub async fn broadcast(clients: Clients, msg: Message) {
    for client in clients.lock().await.values() {
        client.ws.send(Ok(msg.clone())).unwrap();
    }
}
