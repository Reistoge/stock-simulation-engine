use axum::{Router, extract::path, routing::get, routing::post};

use crate::controller::get_info_handler;
use crate::controller::login_handler;

pub fn init() -> Router {
    Router::new()
    .route("/login", post(login_handler))
    .route("/info", get(get_info_handler))
}