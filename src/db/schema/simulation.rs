use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::db::schema::stock::Stock;
/// The stochastic model used to simulate a stock's price path.
///
/// Stored as the `model_type` column (string labels, not a native PG enum) so
/// the table can be filtered and indexed on it cheaply.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, utoipa::ToSchema,
)]
#[column(type = text)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Gbm,
    Merton,
    Ou,
    Heston,
}

impl ModelType {
    /// Derives the matching [`ModelType`] from a [`ModelParams`] variant.
    pub fn from_params(params: &ModelParams) -> Self {
        match params {
            ModelParams::Gbm => Self::Gbm,
            ModelParams::Merton { .. } => Self::Merton,
            ModelParams::Ou { .. } => Self::Ou,
            ModelParams::Heston { .. } => Self::Heston,
        }
    }
}

/// Model-specific simulation parameters, stored as a self-describing JSONB
/// document (e.g. `{"model":"merton","jump_intensity":5.0,...}`).
///
/// Using a tagged enum means a GBM row cannot accidentally hold Merton fields:
/// each variant only exposes the parameters its model understands.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum ModelParams {
    /// Geometric Brownian Motion: no extra parameters.
    Gbm,
    Merton {
        jump_intensity: f64,
        jump_mean: f64,
        jump_volatility: f64,
    },
    Ou {
        mean_reversion: f64,
        reversion_level: f64,
    },
    Heston {
        initial_variance: f64,
        mean_reversion: f64,
        reversion_level: f64,
        vol_of_vol: f64,
        correlation: f64,
    },
}

/// A snapshot of everything the engine needs to replay one simulation run.
///
/// Frozen into the `simulations.parameters` JSONB column at run time so that
/// later edits to the parent stock cannot silently change what a past
/// simulation replays.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimulationParams {
    pub model_type: ModelType,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: ModelParams,
}
#[derive(Debug, toasty::Model)]
#[table = "simulations"]
pub struct Simulation {
    #[key]
    #[auto]
    id: uuid::Uuid,

    /// The stochastic model type used for this simulation.
    model_type: ModelType,

    /// How long the simulation ran, in years (e.g. 1.0 for 1 year).
    time_horizon: f64,

    /// Resolution of the simulated path (e.g. 1000 ticks).
    steps: u32,

    /// The PRNG seed that produced the path; replaying seed + parameters
    /// regenerates the exact same array in the engine.
    random_seed: i64,
    stock_id: Option<uuid::Uuid>,
    #[belongs_to]
    stock: toasty::Deferred<Option<Stock>>,

    /// Frozen copy of the simulation parameters at run time.
    #[column(type = jsonb)]
    parameters: toasty::Json<SimulationParams>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}
