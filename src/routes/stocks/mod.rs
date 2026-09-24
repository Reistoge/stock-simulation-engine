#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::routes::AppState;
use crate::service::simulation::SimulationService;
use crate::service::stock::StockService;

pub mod types;
use types::{CreateStockPayload, UpdateStockPayload, StockQueryFilters, StockResponse};
use crate::routes::simulation::types::SimulationResponse;

fn extract_profile_id(headers: &HeaderMap) -> Result<uuid::Uuid, StatusCode> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)
        .and_then(|token| {
            uuid::Uuid::parse_str(token).map_err(|_| StatusCode::UNAUTHORIZED)
        })
}

/*
    POST STOCKS - Create a new stock for the authenticated user's profile
*/
#[utoipa::path(
    post,
    path = "",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateStockPayload,
    responses(
        (status = 201, description = "Stock created successfully", body = StockResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
async fn post_stocks(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateStockPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;

    let service = StockService::new(app.stock_repository);
    let stock = service
        .create(
            profile_id,
            payload.ticker,
            payload.name,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(StockResponse::from(stock))))
}

/*
    PATCH STOCK - Update a stock
*/
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "The stock UUID")
    ),
    request_body = UpdateStockPayload,
    responses(
        (status = 200, description = "Stock updated successfully", body = StockResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Stock not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn patch_stock(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStockPayload>,
) -> Result<Json<StockResponse>, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;
    let id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let service = StockService::new(app.stock_repository);
    let stock = service
        .update(
            profile_id,
            id,
            payload.name,
        )
        .await?;

    Ok(Json(StockResponse::from(stock)))
}

/*
    GET STOCK BY TICKER
*/
#[utoipa::path(
    get,
    path = "/ticker/{ticker}",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("ticker" = String, Path, description = "The stock ticker symbol")
    ),
    responses(
        (status = 200, description = "Stock retrieved successfully", body = StockResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Stock not found")
    )
)]
async fn get_stock_by_ticker(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(ticker): Path<String>,
) -> Result<Json<StockResponse>, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;

    let service = StockService::new(app.stock_repository);
    let stock = service.get_by_ticker(profile_id, ticker).await?;

    Ok(Json(StockResponse::from(stock)))
}

/*
    GET STOCK BY ID
*/
#[utoipa::path(
    get,
    path = "/id/{id}",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "The stock UUID")
    ),
    responses(
        (status = 200, description = "Stock retrieved successfully", body = StockResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Stock not found")
    )
)]
async fn get_stock_by_id(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<StockResponse>, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;
    let id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let service = StockService::new(app.stock_repository);
    let stock = service.get_by_id(profile_id, id).await?;

    Ok(Json(StockResponse::from(stock)))
}

/*
    LIST STOCKS
*/
#[utoipa::path(
    get,
    path = "",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        StockQueryFilters
    ),
    responses(
        (status = 200, description = "List of stocks", body = [StockResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
async fn list_stocks(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<StockQueryFilters>,
) -> Result<Json<Vec<StockResponse>>, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;

    let service = StockService::new(app.stock_repository);
    let stocks = service
        .list(profile_id, filters.limit, filters.offset)
        .await?;

    Ok(Json(stocks.into_iter().map(StockResponse::from).collect()))
}

/*
    GET SIMULATIONS BY STOCK
*/
#[utoipa::path(
    get,
    path = "/{id}/simulations",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "The stock UUID"),
        ("limit" = Option<u32>, Query, description = "Maximum number of simulations"),
        ("offset" = Option<u32>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of simulations for the stock", body = [SimulationResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Stock not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_stock_simulations(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(filters): Query<StockQueryFilters>,
) -> Result<Json<Vec<SimulationResponse>>, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;
    let stock_id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // First verify the stock belongs to the profile
    let stock_service = StockService::new(app.stock_repository.clone());
    stock_service.get_by_id(profile_id, stock_id).await?;

    // Get simulations for this stock
    let sim_service = SimulationService::new(app.simulation_repository);
    let simulations = sim_service
        .list_by_stock_id(stock_id, filters.limit, filters.offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(simulations.into_iter().map(SimulationResponse::from).collect()))
}

/*
    DELETE STOCK
*/
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "stocks",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "The stock UUID")
    ),
    responses(
        (status = 204, description = "Stock deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Stock not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_stock(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let profile_id = extract_profile_id(&headers)?;
    let id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let service = StockService::new(app.stock_repository);
    service.delete(profile_id, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn init() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(post_stocks))
        .routes(routes!(patch_stock))
        .routes(routes!(get_stock_by_ticker))
        .routes(routes!(get_stock_by_id))
        .routes(routes!(list_stocks))
        .routes(routes!(get_stock_simulations))
        .routes(routes!(delete_stock))
}