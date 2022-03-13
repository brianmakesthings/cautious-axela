use warp::{self, Filter};
mod web_relay;
mod web_requests;


#[tokio::main]
async fn main() {

    let webpage = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./index.html"));

    let public_files = warp::fs::dir("frontend/");
    let routes = webpage
        .or(public_files)
        .with(warp::log("warp::filters::fs"));

    println!("Running at http://0.0.0.0:8000");

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
