use async_trait::async_trait;

use crate::db::schema::stock::Stock;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StockRepository: Send + Sync {
    async fn create(
        &mut self,
        profile_id: uuid::Uuid,
        ticker: &str,
        name: &str,
    ) -> Result<Stock, toasty::Error>;
    async fn find_by_ticker(
        &mut self,
        profile_id: uuid::Uuid,
        ticker: &str,
    ) -> Result<Stock, toasty::Error>;
    async fn find_by_id(
        &mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<Stock, toasty::Error>;
    async fn list_by_profile(
        &mut self,
        profile_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Stock>, toasty::Error>;
    async fn update(
        &mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
        name: Option<String>,
    ) -> Result<Stock, toasty::Error>;
    async fn delete(&mut self, profile_id: uuid::Uuid, id: uuid::Uuid)
    -> Result<(), toasty::Error>;
}

#[derive(Clone)]
pub struct StockRepositoryImpl {
    db: toasty::Db,
}

impl StockRepositoryImpl {
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl StockRepository for StockRepositoryImpl {
    async fn create(
        &mut self,
        profile_id: uuid::Uuid,
        ticker: &str,
        name: &str,
    ) -> Result<Stock, toasty::Error> {
        Stock::create()
            .profile_id(profile_id)
            .ticker(ticker)
            .name(name)
            .exec(&mut self.db)
            .await
    }

    async fn find_by_ticker(
        &mut self,
        profile_id: uuid::Uuid,
        ticker: &str,
    ) -> Result<Stock, toasty::Error> {
        Stock::filter(Stock::fields().profile_id().eq(profile_id))
            .filter(Stock::fields().ticker().eq(ticker))
            .exec(&mut self.db)
            .await?
            .pop()
            .ok_or(toasty::Error::condition_failed("Stock not found"))
    }

    async fn find_by_id(
        &mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<Stock, toasty::Error> {
        Stock::filter(Stock::fields().profile_id().eq(profile_id))
            .filter(Stock::fields().id().eq(id))
            .exec(&mut self.db)
            .await?
            .pop()
            .ok_or(toasty::Error::condition_failed("Stock not found"))
    }

    async fn list_by_profile(
        &mut self,
        profile_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Stock>, toasty::Error> {
        let mut query = Stock::filter(Stock::fields().profile_id().eq(profile_id));

        if let Some(lim) = limit {
            query = query.limit(lim as usize);
        }

        if let Some(off) = offset {
            query = query.offset(off as usize);
        }

        query.exec(&mut self.db).await
    }

    async fn update(
        &mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
        name: Option<String>,
    ) -> Result<Stock, toasty::Error> {
        // First get the stock
        let mut stock = Stock::filter(Stock::fields().profile_id().eq(profile_id))
            .filter(Stock::fields().id().eq(id))
            .exec(&mut self.db)
            .await?
            .pop()
            .ok_or(toasty::Error::condition_failed("Stock not found"))?;

        // Update fields
        if let Some(name) = name {
            stock.name = name;
        }

        // Save the updated stock
        stock.update().exec(&mut self.db).await?;
        Ok(stock)
    }

    async fn delete(
        &mut self,
        profile_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<(), toasty::Error> {
        Stock::filter(Stock::fields().profile_id().eq(profile_id))
            .filter(Stock::fields().id().eq(id))
            .delete()
            .exec(&mut self.db)
            .await
            .map(|_| ())
    }
}
