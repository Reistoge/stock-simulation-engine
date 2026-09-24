use crate::db::schema::stock::Stock;
use crate::repositories::stock::StockRepository;
use axum::http::StatusCode;

pub struct StockService<R: StockRepository> {
    repo: R,
}

impl<R: StockRepository> StockService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        mut self,
        profile_id: uuid::Uuid,
        ticker: String,
        name: String,
    ) -> Result<Stock, StatusCode> {
        // Validate parameters
        if ticker.is_empty() || ticker.len() > 10 {
            return Err(StatusCode::BAD_REQUEST);
        }
        if name.is_empty() || name.len() > 100 {
            return Err(StatusCode::BAD_REQUEST);
        }

        self.repo
            .create(
                profile_id,
                &ticker.to_uppercase(),
                &name,
            )
            .await
            .map_err(|e| {
                eprintln!("Error creating stock: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn get_by_ticker(
        mut self,
        profile_id: uuid::Uuid,
        ticker: String,
    ) -> Result<Stock, StatusCode> {
        self.repo
            .find_by_ticker(profile_id, &ticker.to_uppercase())
            .await
            .map_err(|_| StatusCode::NOT_FOUND)
    }

    pub async fn get_by_id(
        mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<Stock, StatusCode> {
        self.repo
            .find_by_id(profile_id, id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)
    }

    pub async fn list(
        mut self,
        profile_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Stock>, StatusCode> {
        self.repo
            .list_by_profile(profile_id, limit, offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn update(
        mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
        name: Option<String>,
    ) -> Result<Stock, StatusCode> {
        // Validate provided fields
        if let Some(ref name) = name {
            if name.is_empty() || name.len() > 100 {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        self.repo
            .update(profile_id, id, name)
            .await
            .map_err(|e| {
                eprintln!("Error updating stock: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn delete(
        mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<(), StatusCode> {
        self.repo
            .delete(profile_id, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::stock::MockStockRepository;
    use axum::http::StatusCode;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn sample_stock(profile_id: uuid::Uuid) -> Stock {
        Stock {
            id: Uuid::new_v4(),
            name: "Apple Inc.".to_string(),
            ticker: "AAPL".to_string(),
            profile_id: Some(profile_id),
            profile: Default::default(),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }

    #[tokio::test]
    async fn create_stock_success() {
        let profile_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();
        let stock = sample_stock(profile_id);

        repo.expect_create()
            .with(
                eq(profile_id),
                eq("AAPL"),
                eq("Apple Inc."),
            )
            .times(1)
            .returning(move |_, _, _| Ok(stock.clone()));

        let service = StockService::new(repo);
        let result = service
            .create(profile_id, "AAPL".to_string(), "Apple Inc.".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().ticker, "AAPL");
    }

    #[tokio::test]
    async fn create_stock_rejects_empty_ticker() {
        let profile_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();

        repo.expect_create().times(0);

        let service = StockService::new(repo);
        let result = service
            .create(profile_id, "".to_string(), "Apple Inc.".to_string())
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn create_stock_rejects_long_name() {
        let profile_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();

        repo.expect_create().times(0);

        let service = StockService::new(repo);
        let result = service
            .create(profile_id, "AAPL".to_string(), "a".repeat(101))
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn get_by_ticker_not_found() {
        let profile_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();

        repo.expect_find_by_ticker()
            .with(eq(profile_id), eq("AAPL"))
            .times(1)
            .returning(|_, _| Err(toasty::Error::condition_failed("Not found")));

        let service = StockService::new(repo);
        let result = service.get_by_ticker(profile_id, "AAPL".to_string()).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn update_stock_success() {
        let profile_id = Uuid::new_v4();
        let stock_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();
        let stock = sample_stock(profile_id);

        repo.expect_update()
            .with(
                eq(profile_id),
                eq(stock_id),
                eq(Some("Apple Inc. Updated".to_string())),
            )
            .times(1)
            .returning(move |_, _, _| Ok(stock.clone()));

        let service = StockService::new(repo);
        let result = service
            .update(profile_id, stock_id, Some("Apple Inc. Updated".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_stock_rejects_empty_name() {
        let profile_id = Uuid::new_v4();
        let stock_id = Uuid::new_v4();
        let mut repo = MockStockRepository::new();

        repo.expect_update().times(0);

        let service = StockService::new(repo);
        let result = service
            .update(profile_id, stock_id, Some("".to_string()))
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }
}