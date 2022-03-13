use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::filters::ws::Message;

pub type Clients =
    Arc<Mutex<HashMap<String, mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>>>;

pub async fn broadcast(clients: Clients, msg: Message) {
    for sender in clients.lock().await.values() {
        sender.send(Ok(msg.clone())).unwrap();
    }
}
