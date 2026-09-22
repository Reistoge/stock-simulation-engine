use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};


#[derive(Deserialize, ToSchema)]
pub struct CreateStockPayload {
    pub ticker: String,
    pub name: String,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct StockQueryFilters {
    pub market: Option<String>,
}
