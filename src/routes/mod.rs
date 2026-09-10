use axum::Router;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

 
mod user;
mod swagger;
mod stocks;
mod websocket; // standard axum::Router for now
pub fn build_routes() -> Router {

    // 1. Build the OpenApiRouter and nest your OpenAPI-documented modules
    let (router, api) = OpenApiRouter::with_openapi(swagger::ApiDoc::openapi())
        .nest("/stocks",stocks::init())
        .nest("/user", user::init())
        .split_for_parts();
    
    // 2. The `router` is now a standard axum::Router. Merge undocumented routes and SwaggerUI.
    router
        .nest("/ws", websocket::init()) // Use .nest or .merge depending on your websocket router setup
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
}