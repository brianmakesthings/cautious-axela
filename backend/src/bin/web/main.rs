use anyhow::Result;
use core::convert::Infallible;
use futures::FutureExt;
use futures::StreamExt;
use serde::Serialize;
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::filters::ws::Message;
use warp::{self, Filter};
use web_relay::listen_for_web;
use web_requests::{Commands, WebSocketRequest};
use web_ws::{Client, Clients};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;
mod web_relay;
mod web_requests;
mod web_rtp;
mod web_ws;

#[derive(Serialize, Debug)]
struct WsResult {
    id: String,
    response: String,
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let video_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: media_engine::MIME_TYPE_VP8.to_owned(),
            ..Default::default()
        },
        "video".to_owned(),
        "webrtc-rs".to_owned(),
    ));

    let audio_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: media_engine::MIME_TYPE_OPUS.to_owned(),
            ..Default::default()
        },
        "audio".to_owned(),
        "webrtc-rs".to_owned(),
    ));

    let rtc_server_handle_video = web_rtp::mainloop(video_track.clone(), 8002);
    let rtc_server_handle_audio = web_rtp::mainloop(audio_track.clone(), 8004);

    let ws = warp::path("socket")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_track(video_track.clone()))
        .and(with_track(audio_track.clone()))
        .map(|ws: warp::ws::Ws, clients: Clients, video_track: Arc<_>, audio_track: Arc<_>| {
            ws.on_upgrade(move |socket| handle_ws_client(socket, clients, video_track, audio_track))
        });

    let webpage = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("../frontend/index.html"));

    let public_files = warp::fs::dir("../frontend/");
    let routes = webpage
        .or(ws)
        .or(public_files)
        .with(warp::log("warp::filters::fs"));

    println!("Running at http://0.0.0.0:5000");

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;

    rtc_server_handle_video.join().unwrap();
    rtc_server_handle_audio.join().unwrap();
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_track(
    track: Arc<TrackLocalStaticRTP>,
) -> impl Filter<Extract = (Arc<TrackLocalStaticRTP>,), Error = Infallible> + Clone {
    warp::any().map(move || track.clone())
}

async fn handle_ws_client(
    websocket: warp::ws::WebSocket,
    clients: Clients,
    video_track: Arc<TrackLocalStaticRTP>,
    audio_track: Arc<TrackLocalStaticRTP>,
) {
    let (sender, mut receiver) = websocket.split();
    let (ws_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().to_simple().to_string();

    clients.lock().await.insert(
        uuid.clone(),
        Client {
            id: uuid.clone(),
            ws: ws_sender,
        },
    );

    println!("ws connected");
    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error reading message on websocket: {}", e);
                break;
            }
        };

        client_msg(&uuid, msg, &clients, video_track.clone(), audio_track.clone()).await;
    }

    clients.lock().await.remove(&uuid);
    println!("ws disconnected");
}

fn reply(req: WebSocketRequest, client: &Client, msg: String) {
    let response = serde_json::to_string(&WsResult {
        id: req.id,
        response: msg,
    })
    .unwrap();
    client.ws.send(Ok(Message::text(response))).unwrap();
}

// https://github.com/webrtc-rs/examples/tree/main/examples/rtp-to-webrtc
async fn start_rtc(
    req: WebSocketRequest,
    client: &mut Client,
    video_track: Arc<TrackLocalStaticRTP>,
    audio_track: Arc<TrackLocalStaticRTP>,
) {
    println!("Starting rtc with client {}", client.id);
    let mut m = MediaEngine::default();
    m.register_default_codecs().unwrap();

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m).unwrap();

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };
    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

    let rtp_sender_video = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender_video.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    let rtp_sender_audio = peer_connection
        .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();

    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender_audio.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_ice_connection_state_change(Box::new(move |connection_state: RTCIceConnectionState| {
            println!("Connection State has changed {}", connection_state);
            if connection_state == RTCIceConnectionState::Failed {
                println!("Ice Connection failed");
            }
            Box::pin(async {})
        }))
        .await;

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                println!("Peer Connection has gone to failed exiting: Done forwarding");
            }

            Box::pin(async {})
        }))
        .await;

    let desc_data = req.message.clone();
    let offer = from_str::<RTCSessionDescription>(&desc_data).unwrap();

    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await.unwrap();

    // Create an answer
    let answer = peer_connection.create_answer(None).await.unwrap();

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await.unwrap();

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc).unwrap();
        reply(req, client, json_str);
    } else {
        println!("generate local_description failed!");
    }
}

async fn client_msg(
    client_id: &str,
    msg: Message,
    clients: &Clients,
    video_track: Arc<TrackLocalStaticRTP>,
    audio_track: Arc<TrackLocalStaticRTP>,
) {
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    match clients.lock().await.get_mut(client_id) {
        Some(client) => {
            let req: WebSocketRequest = match from_str(message) {
                Ok(req) => req,
                Err(e) => {
                    println!("error parsing request: {}", e);
                    return;
                }
            };

            match req.command {
                Commands::Ping => reply(req, client, "pong".to_string()),
                Commands::DoorGet | Commands::DoorSet => {
                    let res = listen_for_web(req.clone()).await;
                    reply(req, client, res)
                }
                Commands::RtcSession => start_rtc(req, client, video_track, audio_track).await,
                _ => {
                    println!("unhandled command: {}", msg.to_str().unwrap());
                }
            }
        }
        None => return,
    }
}
