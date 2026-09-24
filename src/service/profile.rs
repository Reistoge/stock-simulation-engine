use crate::repositories::profile::ProfileRepository;
use crate::repositories::simulation::SimulationRepository;
use crate::repositories::stock::StockRepository;
use axum::http::StatusCode;
use crate::db::schema::simulation::Simulation;

pub struct ProfileService<P: ProfileRepository, S: SimulationRepository, St: StockRepository> {
    profile_repo: P,
    simulation_repo: S,
    stock_repo: St,
}

impl<P: ProfileRepository, S: SimulationRepository, St: StockRepository> ProfileService<P, S, St> {
    pub fn new(profile_repo: P, simulation_repo: S, stock_repo: St) -> Self {
        Self {
            profile_repo,
            simulation_repo,
            stock_repo,
        }
    }

    pub async fn get_profile_with_data(
        mut self,
        user_id: uuid::Uuid,
    ) -> Result<ProfileWithData, StatusCode> {
        // Get profile
        let profile = self
            .profile_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        // Get stocks for this profile
        let stocks = self
            .stock_repo
            .list_by_profile(profile.id, None, None)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // For each stock, get its simulations
        let mut stocks_with_simulations = Vec::new();
        for stock in stocks {
            let simulations = self
                .simulation_repo
                .list_by_stock_id(stock.id, Some(10), Some(0))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Get model params from first simulation if available
            let (model_type, initial_price, drift, volatility, extra_params) = simulations.first()
                .map(|s| (
                    s.model_type,
                    s.parameters.initial_price,
                    s.parameters.drift,
                    s.parameters.volatility,
                    s.parameters.extra_params,
                ))
                .unwrap_or_else(|| (
                    crate::db::schema::simulation::ModelType::Gbm,
                    0.0, 0.0, 0.0,
                    crate::db::schema::simulation::ModelParams::Gbm,
                ));

            stocks_with_simulations.push(StockWithSimulations {
                id: stock.id,
                ticker: stock.ticker,
                name: stock.name,
                model_type,
                initial_price,
                drift,
                volatility,
                extra_params,
                simulations: simulations.into_iter().map(SimulationBasic::from).collect(),
            });
        }

        Ok(ProfileWithData {
            id: profile.id,
            user_id: profile.user_id.unwrap_or_default(),
            created_at: profile.created_at.to_string(),
            updated_at: profile.updated_at.to_string(),
            stocks: stocks_with_simulations,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ProfileWithData {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub created_at: String,
    pub updated_at: String,
    pub stocks: Vec<StockWithSimulations>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct StockWithSimulations {
    pub id: uuid::Uuid,
    pub ticker: String,
    pub name: String,
    pub model_type: crate::db::schema::simulation::ModelType,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: crate::db::schema::simulation::ModelParams,
    pub simulations: Vec<SimulationBasic>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SimulationBasic {
    pub id: uuid::Uuid,
    pub model_type: crate::db::schema::simulation::ModelType,
    pub time_horizon: f64,
    pub steps: u32,
    pub random_seed: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Simulation> for SimulationBasic {
    fn from(sim: Simulation) -> Self {
        Self {
            id: sim.id,
            model_type: sim.model_type,
            time_horizon: sim.time_horizon,
            steps: sim.steps,
            random_seed: sim.random_seed,
            created_at: sim.created_at.to_string(),
            updated_at: sim.updated_at.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::simulation::{ModelParams, ModelType, Simulation, SimulationParams};
    use crate::db::schema::stock::Stock;
    use crate::db::schema::profile::Profile;
    use crate::repositories::profile::MockProfileRepository;
    use crate::repositories::simulation::MockSimulationRepository;
    use crate::repositories::stock::MockStockRepository;
    use axum::http::StatusCode;
    use mockall::predicate::eq;
    use toasty::Json;
    use uuid::Uuid;

    fn sample_simulation(stock_id: uuid::Uuid) -> Simulation {
        Simulation {
            id: Uuid::new_v4(),
            model_type: ModelType::Gbm,
            time_horizon: 1.0,
            steps: 1000,
            random_seed: 42,
            stock_id: Some(stock_id),
            stock: Default::default(),
            parameters: Json(SimulationParams {
                initial_price: 150.0,
                drift: 0.05,
                volatility: 0.2,
                extra_params: ModelParams::Gbm,
            }),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }

    #[tokio::test]
    async fn get_profile_with_data_success() {
        let user_id = Uuid::new_v4();
        let profile_id = Uuid::new_v4();
        // Use a fixed stock_id for consistency
        let stock_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let mut profile_repo = MockProfileRepository::new();
        let mut simulation_repo = MockSimulationRepository::new();
        let mut stock_repo = MockStockRepository::new();

        let profile = Profile {
            id: profile_id,
            user_id: Some(user_id),
            user: Default::default(),
            stocks: vec![].into(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        // Create stock with the fixed stock_id (no model_type, drift, etc. anymore)
        let stock = Stock {
            id: stock_id,
            name: "Apple Inc.".to_string(),
            ticker: "AAPL".to_string(),
            profile_id: Some(profile_id),
            profile: Default::default(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let simulation = sample_simulation(stock_id);

        profile_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(profile.clone()));

        stock_repo
            .expect_list_by_profile()
            .with(eq(profile_id), eq(None), eq(None))
            .times(1)
            .returning(move |_, _, _| Ok(vec![stock.clone()]));

        simulation_repo
            .expect_list_by_stock_id()
            .with(eq(stock_id), eq(Some(10)), eq(Some(0)))
            .times(1)
            .returning(move |_, _, _| Ok(vec![simulation.clone()]));

        let service = ProfileService::new(profile_repo, simulation_repo, stock_repo);
        let result = service.get_profile_with_data(user_id).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.id, profile_id);
        assert_eq!(data.stocks.len(), 1);
        assert_eq!(data.stocks[0].ticker, "AAPL");
        assert_eq!(data.stocks[0].simulations.len(), 1);
    }

    #[tokio::test]
    async fn get_profile_with_data_not_found() {
        let user_id = Uuid::new_v4();

        let mut profile_repo = MockProfileRepository::new();
        let simulation_repo = MockSimulationRepository::new();
        let stock_repo = MockStockRepository::new();

        profile_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(toasty::Error::condition_failed("Not found")));

        let service = ProfileService::new(profile_repo, simulation_repo, stock_repo);
        let result = service.get_profile_with_data(user_id).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }
}