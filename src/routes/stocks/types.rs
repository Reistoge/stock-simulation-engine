use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema)]
pub struct CreateStockPayload {
    pub ticker: String,
    pub price: f64,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct StockQueryFilters {
    pub market: Option<String>,
}
