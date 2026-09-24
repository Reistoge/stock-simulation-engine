#![cfg_attr(coverage_nightly, coverage(off))]
use axum::Router;

use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::repositories::user::UserRepositoryImpl;
use crate::repositories::simulation::SimulationRepositoryImpl;
use crate::repositories::profile::ProfileRepositoryImpl;
use crate::repositories::stock::StockRepositoryImpl;

mod user;
mod swagger;
mod stocks;
mod simulation;
mod profile;
mod websocket; // standard axum::Router for now

// Shared application state. Repositories are injected into the handlers
// instead of handing them the raw `toasty::Db`.
#[derive(Clone)]
pub struct AppState {
    pub user_repository: UserRepositoryImpl,
    pub profile_repository: ProfileRepositoryImpl,
    pub simulation_repository: SimulationRepositoryImpl,
    pub stock_repository: StockRepositoryImpl,
}

pub fn build_routes(app_state: AppState) -> Router {

    // 1. Build the OpenApiRouter and nest your OpenAPI-documented modules
    let (router, api) = OpenApiRouter::<AppState>::with_openapi(swagger::ApiDoc::openapi())
        .nest("/stocks", stocks::init())
        .nest("/user", user::init())
        .nest("/simulations", simulation::init())
        .nest("/profile", profile::init())
        .split_for_parts();

    // 2. The `router` is now a standard axum::Router. Merge undocumented routes and SwaggerUI.
    router
        .nest("/ws", websocket::init()) // Use .nest or .merge depending on your websocket router setup
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        // Allow the Angular dev server to call this API from a different origin.
        // `Access-Control-Allow-Headers: *` (CorsLayer::permissive()) does NOT
        // cover `Authorization` per the CORS spec — it must be listed explicitly,
        // or authenticated requests (e.g. GET /profile) get blocked by the browser.
        // TODO: restrict allow_origin to your real frontend origin(s) before deploying.
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        // 3. Provide the shared `AppState` (with its repositories) to every handler's
        // `State<AppState>` extractor, and convert `Router<AppState>` into the
        // stateless `Router<()>` axum::serve expects.
        .with_state(app_state)
}