

use axum::{Router, routing::{any, get}};


mod stocks;
mod websocket; 


pub fn build_routes() -> Router {
    Router::new()
        .merge(Router::new().route("/", get(|| async { "Hello, World!" })))
        .merge(stocks::init())
        .merge(websocket::init())
}

