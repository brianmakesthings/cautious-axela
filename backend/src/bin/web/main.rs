use warp::{self, Filter};
use std::{thread, sync::mpsc};
mod web_relay;
mod web_requests;

#[tokio::main]
async fn main() {

    let (tx, _rx) = mpsc::channel::<String>();
    let tcp_listener_handle = thread::spawn(|| {web_relay::listen_for_intercom(tx)});

    let webpage = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("frontend/index.html"));

    let public_files = warp::fs::dir("");
    let routes = webpage
        .or(public_files)
        .with(warp::log("warp::filters::fs"));

    println!("Running at http://0.0.0.0:8000");

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;

    tcp_listener_handle.join().unwrap();
}
