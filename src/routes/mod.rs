use axum::{Router, routing::get};

mod stocks;

pub fn build_routes() -> Router {
    Router::new()
        .merge(Router::new().route("/", get(|| async { "Hello, World!" })))
        .merge(stocks::init())
}
