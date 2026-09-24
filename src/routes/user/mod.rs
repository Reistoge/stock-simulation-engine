#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::types::{LoginInfo, LoginResponse, RegisterInfo, RegisterResponse};
use crate::auth::validation::extract_bearer_token;
use crate::routes::AppState;
use crate::service::user::UserService;

/*
 POST USER LOGIN
*/
#[utoipa::path(
    post,
    path ="/login",
    tag = "user",
    request_body = LoginInfo,
    responses(
        (status = 200, description = "List all stocks successfully", body = [LoginResponse])
    )
)]
pub async fn login(
    State(app): State<AppState>,
    Json(login_info): Json<LoginInfo>,
) -> Result<Json<LoginResponse>, StatusCode> {
    println!("Hit the POST login controller");
    let service = UserService::new(app.user_repository);
    let r = service.login(login_info).await?;
    Ok(Json(r))
}

/*
 GET USER INFO
*/
#[utoipa::path(
    get,
    path ="/info",
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User info retrieved successfully", body = String)
    )
)]
pub async fn get_info(
    State(app): State<AppState>,
    header_map: HeaderMap,
) -> Result<Json<String>, StatusCode> {
    println!("Hit the get info controller");
    let token = extract_bearer_token(&header_map)?;

    let service = UserService::new(app.user_repository);
    let info = service.get_info(&token).await?;
    Ok(Json(info))
}

/*
 POST USER REGISTER
*/
#[utoipa::path(
    post,
    path ="/register",
    tag = "user",
    request_body = RegisterInfo,
    responses(
        (status = 200, description = "User registered successfully", body = [RegisterResponse])
    )
)]
pub async fn register(
    State(app): State<AppState>,
    Json(register_info): Json<RegisterInfo>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    println!("Hit the POST register controller");
    let service = UserService::new(app.user_repository);
    let r = service.register(register_info).await?;
    Ok(Json(r))
}

pub fn init() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(get_info))
        .routes(routes!(register))
}