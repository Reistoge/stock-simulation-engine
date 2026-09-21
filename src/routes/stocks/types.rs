use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::db::schema::stock::{ModelParams, ModelType};

#[derive(Deserialize, ToSchema)]
pub struct CreateStockPayload {
    pub ticker: String,
    pub name: String,
    pub model_type: ModelType,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: ModelParams,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct StockQueryFilters {
    pub market: Option<String>,
}
