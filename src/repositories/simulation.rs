use async_trait::async_trait;

use crate::db::schema::simulation::{ModelType, Simulation, SimulationParams};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SimulationRepository: Send + Sync {
    async fn find_by_id(&mut self, id: uuid::Uuid) -> Result<Simulation, toasty::Error>;
    async fn list(
        &mut self,
        model_type: Option<ModelType>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, toasty::Error>;
    async fn list_by_stock_id(
        &mut self,
        stock_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, toasty::Error>;
    async fn create(
        &mut self,
        model_type: ModelType,
        time_horizon: f64,
        steps: u32,
        random_seed: i64,
        stock_id: Option<uuid::Uuid>,
        parameters: SimulationParams,
    ) -> Result<Simulation, toasty::Error>;
    async fn delete(&mut self, id: uuid::Uuid) -> Result<(), toasty::Error>;
}

#[derive(Clone)]
pub struct SimulationRepositoryImpl {
    db: toasty::Db,
}

impl SimulationRepositoryImpl {
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SimulationRepository for SimulationRepositoryImpl {
    async fn find_by_id(&mut self, id: uuid::Uuid) -> Result<Simulation, toasty::Error> {
        Simulation::get_by_id(&mut self.db, id).await
    }

    async fn list(
        &mut self,
        model_type: Option<ModelType>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, toasty::Error> {
        let mut query = if let Some(mt) = model_type {
            Simulation::filter(Simulation::fields().model_type().eq(mt))
        } else {
            Simulation::all()
        };
        
        if let Some(lim) = limit {
            query = query.limit(lim as usize);
        }
        
        if let Some(off) = offset {
            query = query.offset(off as usize);
        }
        
        query.exec(&mut self.db).await
    }

    async fn list_by_stock_id(
        &mut self,
        stock_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, toasty::Error> {
        let mut query = Simulation::filter(Simulation::fields().stock_id().eq(stock_id));
        
        if let Some(lim) = limit {
            query = query.limit(lim as usize);
        }
        
        if let Some(off) = offset {
            query = query.offset(off as usize);
        }
        
        query.exec(&mut self.db).await
    }

    async fn create(
        &mut self,
        model_type: ModelType,
        time_horizon: f64,
        steps: u32,
        random_seed: i64,
        stock_id: Option<uuid::Uuid>,
        parameters: SimulationParams,
    ) -> Result<Simulation, toasty::Error> {
        let mut builder = Simulation::create()
            .model_type(model_type)
            .time_horizon(time_horizon)
            .steps(steps)
            .random_seed(random_seed)
            .parameters(toasty::Json(parameters));
        
        if let Some(sid) = stock_id {
            builder = builder.stock_id(sid);
        }
        
        builder.exec(&mut self.db).await
    }

    async fn delete(&mut self, id: uuid::Uuid) -> Result<(), toasty::Error> {
        Simulation::delete_by_id(&mut self.db, id).await.map(|_| ())
    }
}