use axum::Router;

use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

 
mod user;
mod swagger;
mod stocks;
mod websocket; // standard axum::Router for now
pub fn build_routes(db: toasty::Db) -> Router {

    // 1. Build the OpenApiRouter and nest your OpenAPI-documented modules
    let (router, api) = OpenApiRouter::<toasty::Db>::with_openapi(swagger::ApiDoc::openapi())
        .nest("/stocks",stocks::init())
        .nest("/user", user::init())
        .split_for_parts();

    // 2. The `router` is now a standard axum::Router. Merge undocumented routes and SwaggerUI.
    router
        .nest("/ws", websocket::init()) // Use .nest or .merge depending on your websocket router setup
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        // Allow the Angular dev server to call this API from a different origin.
        // TODO: restrict this to your real frontend origin(s) before deploying.
        .layer(CorsLayer::permissive())
        // 3. Provide the shared `Db` handle to every handler's `State<Db>` extractor,
        // and convert `Router<Db>` into the stateless `Router<()>` axum::serve expects.
        .with_state(db)
}