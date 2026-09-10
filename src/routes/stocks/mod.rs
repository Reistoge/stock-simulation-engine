use axum::{
    Json,
    extract::{Path, Query},
    http::HeaderMap,
};
// stocks.rs
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod types;
use types::{CreateStockPayload, StockQueryFilters};
/*
    GET STOCKS
*/
#[utoipa::path(
    get,
    path ="/{ticker}",
    tag = "stocks",
    params(
        StockQueryFilters,
        ("ticker" = String, Path, description = "The stock ticker symbol"),
        ("authorization" = String, Header, description = "Bearer token") // Document header
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
    request_body = CreateStockPayload, // Document the JSON body
    responses(
        (status = 201, description = "Create a stock successfully")
    )
)]
async fn post_stocks(Json(payload): Json<CreateStockPayload>) -> String {
    println!("Hit the post_stocks Controller");
    format!("Created stock {} at ${}", payload.ticker, payload.price)
}

pub fn init() -> OpenApiRouter {
    // routes! macro handles multiple HTTP methods on the same path automatically
    OpenApiRouter::new().routes(routes!(get_stocks, post_stocks))
}
