#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::{State},
    http::{HeaderMap, StatusCode},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::routes::AppState;
use crate::service::profile::ProfileService;
use crate::auth::validation::{extract_bearer_token, extract_user_id_from_token};

pub mod types;
use types::{ProfileWithData};

/*
    GET PROFILE - Get authenticated user's profile with stocks and simulations
*/
#[utoipa::path(
    get,
    path = "",
    tag = "profile",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ProfileWithData),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found")
    )
)]
async fn get_profile(
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ProfileWithData>, StatusCode> {
    let user_id = extract_user_id(&headers)?;

    let service = ProfileService::new(
        app.profile_repository,
        app.simulation_repository,
        app.stock_repository,
    );
    let profile = service.get_profile_with_data(user_id).await?;

    Ok(Json(profile))
}

fn extract_user_id(headers: &HeaderMap) -> Result<uuid::Uuid, StatusCode> {
    let token = extract_bearer_token(headers)?;
    extract_user_id_from_token(&token)
}

pub fn init() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_profile))
}