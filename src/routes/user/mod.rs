#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::{
    types::{LoginInfo, LoginResponse, RegisterInfo, RegisterResponse},
    validation::{get_info_handler, login_handler, register_handler},
};
use crate::routes::AppState;
/*
 POST USER LOGIN
*/
#[utoipa::path(
    post,
    path ="/login",
    tag = "user",
    params(
        ("authorization" = String, Header, description = "Bearer token") // Document header
    ),
    request_body = LoginInfo,
    responses(
        (status = 200, description = "List all stocks successfully", body = [LoginResponse])
    )
)]
pub async fn login(
    State(mut app): State<AppState>,
    Json(login_info): Json<LoginInfo>,
) -> Result<Json<LoginResponse>, StatusCode> {
    println!("Hit the POST login controller");
    let r = match login_handler(&mut app.user_repository, Json(login_info)).await {
        Ok(r) => r,
        Err(err) => {
            return Err(err.into());
        }
    };
    Ok(r)
}

/*
 GET USER INFO
*/
#[utoipa::path(
    get,
    path ="/info",
    tag = "user",
    params(
        ("authorization" = String, Header, description = "Bearer token") // Document header
    ),
    responses(
        (status = 200, description = "List all stocks successfully", body = [LoginInfo])
    )
)]
pub async fn get_info(header_map: HeaderMap) -> Result<Json<String>, StatusCode> {
    println!("Hit the get info controller");
    let r = match get_info_handler(header_map).await {
        Ok(r) => r,
        Err(err) => {
            return Err(err.into());
        }
    };
    Ok(r)
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
    State(mut app): State<AppState>,
    Json(register_info): Json<RegisterInfo>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    println!("Hit the POST register controller");
    let r = match register_handler(&mut app.user_repository, Json(register_info)).await {
        Ok(r) => r,
        Err(err) => {
            return Err(err.into());
        }
    };
    Ok(r)
}

pub fn init() -> OpenApiRouter<AppState> {
    // routes! groups handlers that share the SAME path but different HTTP
    // methods. login/get_info/register are on different paths, so each needs
    // its own .routes(routes!(...)) call — merging them together here caused
    // both POST handlers (login, register) to collide in one method router.
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(get_info))
        .routes(routes!(register))
}
