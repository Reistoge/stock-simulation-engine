#![cfg_attr(coverage_nightly, coverage(off))]
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::routes::AppState;
use crate::routes::simulation::types::{
    CreateSimulationPayload, SimulationQueryFilters, SimulationResponse, SimulationParamsPayload,
};
use crate::service::simulation::SimulationService;

pub mod types;

/*
    POST SIMULATIONS
*/
#[utoipa::path(
    post,
    path = "",
    tag = "simulations",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateSimulationPayload,
    responses(
        (status = 201, description = "Simulation created successfully", body = SimulationResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create_simulation(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateSimulationPayload>,
) -> Result<Json<SimulationResponse>, StatusCode> {
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    if _auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let params: SimulationParamsPayload = SimulationParamsPayload {
        initial_price: payload.initial_price,
        drift: payload.drift,
        volatility: payload.volatility,
        extra_params: payload.extra_params,
    };

    let service = SimulationService::new(app.simulation_repository);
    let simulation = service
        .create(
            payload.model_type,
            payload.time_horizon,
            payload.steps,
            payload.random_seed,
            payload.stock_id,
            params.into(),
        )
        .await?;

    Ok(Json(simulation.into()))
}

/*
    GET SIMULATION BY ID
*/
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "simulations",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    responses(
        (status = 200, description = "Simulation retrieved successfully", body = SimulationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Simulation not found")
    )
)]
async fn get_simulation(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SimulationResponse>, StatusCode> {
    let id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    if _auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let service = SimulationService::new(app.simulation_repository);
    let simulation = service.get_by_id(id).await?;

    Ok(Json(simulation.into()))
}

/*
    LIST SIMULATIONS
*/
#[utoipa::path(
    get,
    path = "",
    tag = "simulations",
    security(
        ("bearer_auth" = [])
    ),
    params(
        SimulationQueryFilters
    ),
    responses(
        (status = 200, description = "List of simulations", body = [SimulationResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
async fn list_simulations(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(filters): Query<SimulationQueryFilters>,
) -> Result<Json<Vec<SimulationResponse>>, StatusCode> {
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    if _auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let service = SimulationService::new(app.simulation_repository);
    let simulations = service
        .list(filters.model_type, filters.limit, filters.offset)
        .await?;

    Ok(Json(simulations.into_iter().map(|s| s.into()).collect()))
}

/*
    DELETE SIMULATION
*/
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "simulations",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    responses(
        (status = 204, description = "Simulation deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Simulation not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_simulation(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    if _auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let service = SimulationService::new(app.simulation_repository);
    service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn init() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_simulation))
        .routes(routes!(get_simulation))
        .routes(routes!(list_simulations))
        .routes(routes!(delete_simulation))
}