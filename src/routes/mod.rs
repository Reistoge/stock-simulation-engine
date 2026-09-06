

use axum::{Router, routing::{any, get}};


mod stocks;
mod websocket;
mod users; 


pub fn build_routes() -> Router {
    Router::new()
        .merge(Router::new().route("/", get(|| async { "Hello, World!" })))
        .merge(stocks::init())
        .merge(websocket::init())
        .merge(users::init())
}

