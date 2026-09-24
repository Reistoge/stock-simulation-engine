use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::db::schema::simulation::{ModelParams, ModelType};

#[derive(Deserialize, ToSchema)]
pub struct CreateSimulationPayload {
    pub model_type: ModelType,
    pub time_horizon: f64,
    pub steps: u32,
    pub random_seed: i64,
    pub stock_id: Option<uuid::Uuid>,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: ModelParams,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SimulationParamsPayload {
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: ModelParams,
}

impl From<SimulationParamsPayload> for crate::db::schema::simulation::SimulationParams {
    fn from(payload: SimulationParamsPayload) -> Self {
        Self {
            initial_price: payload.initial_price,
            drift: payload.drift,
            volatility: payload.volatility,
            extra_params: payload.extra_params,
        }
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SimulationQueryFilters {
    pub model_type: Option<ModelType>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct SimulationResponse {
    pub id: uuid::Uuid,
    pub model_type: ModelType,
    pub time_horizon: f64,
    pub steps: u32,
    pub random_seed: i64,
    pub stock_id: Option<uuid::Uuid>,
    pub parameters: SimulationParamsPayload,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::db::schema::simulation::Simulation> for SimulationResponse {
    fn from(sim: crate::db::schema::simulation::Simulation) -> Self {
        let params: SimulationParamsPayload = SimulationParamsPayload {
            initial_price: sim.parameters.initial_price,
            drift: sim.parameters.drift,
            volatility: sim.parameters.volatility,
            extra_params: sim.parameters.extra_params,
        };
        
        Self {
            id: sim.id,
            model_type: sim.model_type,
            time_horizon: sim.time_horizon,
            steps: sim.steps,
            random_seed: sim.random_seed,
            stock_id: sim.stock_id,
            parameters: params,
            created_at: sim.created_at.to_string(),
            updated_at: sim.updated_at.to_string(),
        }
    }
}