use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::db::schema::stock::Stock;

#[derive(Deserialize, ToSchema)]
pub struct CreateStockPayload {
    pub ticker: String,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateStockPayload {
    pub name: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct StockQueryFilters {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct StockResponse {
    pub id: uuid::Uuid,
    pub ticker: String,
    pub name: String,
    pub profile_id: Option<uuid::Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Stock> for StockResponse {
    fn from(stock: Stock) -> Self {
        Self {
            id: stock.id,
            ticker: stock.ticker,
            name: stock.name,
            profile_id: stock.profile_id,
            created_at: stock.created_at.to_string(),
            updated_at: stock.updated_at.to_string(),
        }
    }
}