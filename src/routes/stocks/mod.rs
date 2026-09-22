#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
// stocks.rs
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::routes::AppState;

pub mod types;
use types::{CreateStockPayload, StockQueryFilters};
/*
    GET STOCKS
*/
#[utoipa::path(
    get,
    path ="/{ticker}",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        StockQueryFilters,
        ("ticker" = String, Path, description = "The stock ticker symbol")
    ),
    responses(
        (status = 200, description = "List all stocks successfully")
    )
)]
async fn get_stocks(
    Path(ticker): Path<String>,
    Query(filters): Query<StockQueryFilters>,
    headers: HeaderMap,
) -> String {
    println!("Hit the get_stocks Controller");
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    format!("Ticker: {}, Market: {:?}", ticker, filters.market)
}
/*
    POST STOCKS
*/
#[utoipa::path(
    post,
    path = "",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateStockPayload, // Document the JSON body
    responses(
        (status = 201, description = "Create a stock successfully"),
        (status = 400, description = "model_type does not match extra_params")
    )
)]
async fn post_stocks(
    headers: HeaderMap,
    Json(payload): Json<CreateStockPayload>,
) -> Result<String, StatusCode> {
    println!("Hit the post_stocks Controller");
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    if _auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(format!(
        "Created stock {} ({}) at ${} (drift {}, vol {})",
        payload.ticker,
        payload.name,
        payload.initial_price,
        payload.drift,
        payload.volatility,
    ))
}

pub fn init() -> OpenApiRouter<AppState> {
    // routes! macro handles multiple HTTP methods on the same path automatically
    OpenApiRouter::new().routes(routes!(get_stocks, post_stocks))
}