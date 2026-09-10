use axum::{
    Json,
    http::{HeaderMap, StatusCode},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::{
    types::{LoginInfo, LoginResponse},
    validation::{get_info_handler, login_handler},
};
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
pub async fn login(Json(login_info): Json<LoginInfo>) -> Result<Json<LoginResponse>, StatusCode> {
    println!("Hit the POST login controller");
    let r = match login_handler(Json(login_info)).await {
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

pub fn init() -> OpenApiRouter {
    // routes! macro handles multiple HTTP methods on the same path automatically
    OpenApiRouter::new().routes(routes!(login, get_info))
}
