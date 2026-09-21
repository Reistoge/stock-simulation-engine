use crate::db::schema::stock::{SimulationParams, Stock};
use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
#[table = "simulations"]
pub struct Simulation {
    #[key]
    #[auto]
    id: uuid::Uuid,

    #[index]
    stock_id: uuid::Uuid,
    #[belongs_to]
    stock: toasty::Deferred<Stock>,

    /// How long the simulation ran, in years (e.g. 1.0 for 1 year).
    time_horizon: f64,

    /// Resolution of the simulated path (e.g. 1000 ticks).
    steps: u32,

    /// The PRNG seed that produced the path; replaying seed + parameters
    /// regenerates the exact same array in the engine.
    random_seed: i64,

    /// Frozen copy of the stock's parameters at run time, so a replayed
    /// simulation is deterministic even if the stock is edited later.
    #[column(type = jsonb)]
    parameters: toasty::Json<SimulationParams>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}