#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::{State},
    http::{HeaderMap, StatusCode},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::routes::AppState;
use crate::service::profile::ProfileService;

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
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)
        .and_then(|token| {
            // For now, we'll parse the token as a UUID (simplified)
            // In a real implementation, this would validate the JWT and extract user_id
            uuid::Uuid::parse_str(token).map_err(|_| StatusCode::UNAUTHORIZED)
        })
}

pub fn init() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_profile))
}