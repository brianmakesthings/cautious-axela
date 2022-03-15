use core::convert::Infallible;
use futures::FutureExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::filters::ws::Message;
use warp::{self, Filter};
mod web_udp;
mod web_ws;
use web_ws::Clients;

#[derive(Deserialize, Debug)]
struct WsRequest {
    id: String,
    command: String,
    message: String,
}

#[derive(Serialize, Debug)]
struct WsResult {
    id: String,
    response: String,
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let udp_server_handle = web_udp::mainloop(clients.clone());

    let ws = warp::path("socket")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients: Clients| {
            ws.on_upgrade(move |socket| handle_ws_client(socket, clients))
        });

    let webpage = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("../frontend/index.html"));

    let public_files = warp::fs::dir("../frontend");
    let routes = webpage
        .or(ws)
        .or(public_files)
        .with(warp::log("warp::filters::fs"));

    println!("Running at http://0.0.0.0:8000");

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;

    udp_server_handle.join().unwrap();
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn handle_ws_client(websocket: warp::ws::WebSocket, clients: Clients) {
    let (sender, mut receiver) = websocket.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().to_simple().to_string();
    clients.lock().await.insert(uuid.clone(), client_sender);

    println!("ws connected");
    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error reading message on websocket: {}", e);
                break;
            }
        };

        client_msg(&uuid, msg, &clients).await;
    }

    clients.lock().await.remove(&uuid);
    println!("ws disconnected");
}

fn reply(
    req: WsRequest,
    sender: &mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
    msg: String,
) {
    let response = serde_json::to_string(&WsResult {
        id: req.id,
        response: msg,
    })
    .unwrap();
    sender.send(Ok(Message::text(response))).unwrap();
}

async fn client_msg(client_id: &str, msg: Message, clients: &Clients) {
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    match clients.lock().await.get(client_id) {
        Some(sender) => {
            println!("{}", message);
            let req: WsRequest = serde_json::from_str(message).unwrap();
            match req.command.as_str() {
                "ping" => {
                    reply(req, sender, "pong".to_string());
                }
                "lock" => {
                }
                _ => {
                    println!("unhandled command: {} {}", req.command, req.message);
                }
            }
        }
        None => return,
    }
}
