use crate::db::schema::simulation::{ModelType, Simulation, SimulationParams};
use crate::repositories::simulation::SimulationRepository;
use axum::http::StatusCode;

pub struct SimulationService<R: SimulationRepository> {
    repo: R,
}

impl<R: SimulationRepository> SimulationService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_by_id(mut self, id: uuid::Uuid) -> Result<Simulation, StatusCode> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)
    }

    pub async fn list(
        mut self,
        model_type: Option<ModelType>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, StatusCode> {
        self.repo
            .list(model_type, limit, offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn list_by_stock_id(
        mut self,
        stock_id: uuid::Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Simulation>, StatusCode> {
        self.repo
            .list_by_stock_id(stock_id, limit, offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
     
    pub async fn create(
        mut self,
        model_type: ModelType,
        time_horizon: f64,
        steps: u32,
        random_seed: i64,
        stock_id: Option<uuid::Uuid>,
        parameters: SimulationParams,
    ) -> Result<Simulation, StatusCode> {
        // Validate parameters
        if time_horizon <= 0.0 {
            return Err(StatusCode::BAD_REQUEST);
        }
        if steps == 0 {
            return Err(StatusCode::BAD_REQUEST);
        }
        if parameters.initial_price <= 0.0 {
            return Err(StatusCode::BAD_REQUEST);
        }
        if parameters.volatility < 0.0 {
            return Err(StatusCode::BAD_REQUEST);
        }

        self.repo
            .create(
                model_type,
                time_horizon,
                steps,
                random_seed,
                stock_id,
                parameters,
            )
            .await
            .map_err(|e| {
                eprintln!("Error creating simulation: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn delete(mut self, id: uuid::Uuid) -> Result<(), StatusCode> {
        self.repo
            .delete(id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::simulation::{ModelParams, ModelType, SimulationParams};
    use crate::repositories::simulation::MockSimulationRepository;
    use axum::http::StatusCode;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn sample_params() -> SimulationParams {
        SimulationParams {
            initial_price: 100.0,
            drift: 0.05,
            volatility: 0.2,
            extra_params: ModelParams::Gbm,
        }
    }

    #[tokio::test]
    async fn create_simulation_success() {
        let mut repo = MockSimulationRepository::new();
        let sim_id = Uuid::new_v4();
        let params = sample_params();

        repo.expect_create()
            .with(
                eq(ModelType::Gbm),
                eq(1.0),
                eq(1000),
                eq(42),
                eq(None),
                eq(params),
            )
            .times(1)
            .returning(move |_, _, _, _, _, _| {
                Ok(Simulation {
                    id: sim_id,
                    model_type: ModelType::Gbm,
                    time_horizon: 1.0,
                    steps: 1000,
                    random_seed: 42,
                    stock_id: None,
                    stock: Default::default(),
                    parameters: toasty::Json(params),
                    created_at: Default::default(),
                    updated_at: Default::default(),
                })
            });

        let service = SimulationService::new(repo);
        let result = service
            .create(ModelType::Gbm, 1.0, 1000, 42, None, params)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, sim_id);
    }

    #[tokio::test]
    async fn create_simulation_rejects_invalid_time_horizon() {
        let mut repo = MockSimulationRepository::new();
        let params = sample_params();

        repo.expect_create().times(0);

        let service = SimulationService::new(repo);
        let result = service
            .create(ModelType::Gbm, 0.0, 1000, 42, None, params)
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn create_simulation_rejects_zero_steps() {
        let mut repo = MockSimulationRepository::new();
        let params = sample_params();

        repo.expect_create().times(0);

        let service = SimulationService::new(repo);
        let result = service
            .create(ModelType::Gbm, 1.0, 0, 42, None, params)
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn create_simulation_rejects_negative_price() {
        let mut repo = MockSimulationRepository::new();
        let mut params = sample_params();
        params.initial_price = -100.0;

        repo.expect_create().times(0);

        let service = SimulationService::new(repo);
        let result = service
            .create(ModelType::Gbm, 1.0, 1000, 42, None, params)
            .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn get_by_id_not_found() {
        let mut repo = MockSimulationRepository::new();
        let test_id = Uuid::new_v4();

        repo.expect_find_by_id()
            .with(eq(test_id))
            .times(1)
            .returning(|_| Err(toasty::Error::condition_failed("Not found")));

        let service = SimulationService::new(repo);
        let result = service.get_by_id(test_id).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn list_by_stock_id_success() {
        let mut repo = MockSimulationRepository::new();
        let stock_id = Uuid::new_v4();
        let sim_id = Uuid::new_v4();
        let params = sample_params();

        let simulation = Simulation {
            id: sim_id,
            model_type: ModelType::Gbm,
            time_horizon: 1.0,
            steps: 1000,
            random_seed: 42,
            stock_id: Some(stock_id),
            stock: Default::default(),
            parameters: toasty::Json(params),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        repo.expect_list_by_stock_id()
            .with(eq(stock_id), eq(Some(10)), eq(Some(0)))
            .times(1)
            .returning(move |_, _, _| Ok(vec![simulation.clone()]));

        let service = SimulationService::new(repo);
        let result = service.list_by_stock_id(stock_id, Some(10), Some(0)).await;

        assert!(result.is_ok());
        let sims = result.unwrap();
        assert_eq!(sims.len(), 1);
        assert_eq!(sims[0].id, sim_id);
    }
}
