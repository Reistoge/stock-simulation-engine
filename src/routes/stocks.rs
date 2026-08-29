use axum::{Router, routing::get};

pub fn init() -> Router {
    Router::new().route(
        "/stocks",
        get(|| async { "GET stocks route!" })
            .post(|| async { "POST stocks route !" })
            .delete(|| async { "DELETE stocks route !" })
            .patch(|| async { "PATCH stocks route !" })
            .put(|| async { "PUT stocks route !" }),
    )
}
