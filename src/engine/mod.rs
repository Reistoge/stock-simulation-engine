//! Deterministic SDE replay engine (strategy pattern).
//!
//! Why this exists: `Simulation` rows store only replay config
//! (seed + frozen parameters), never the path itself, so every read
//! regenerates the exact same ticks array from the seed.

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};

use RustQuant::stochastics::{
    GeometricBrownianMotion, MertonJumpDiffusion, OrnsteinUhlenbeck, StochasticProcess,
};

use crate::db::schema::simulation::{ModelParams, ModelType, Simulation, SimulationParams};

/// Upper bound for a single ticks response to avoid OOM on large stored sims.
pub const MAX_TICKS_STEPS: u32 = 100_000;

/// Strategy for generating one price path (ticks) from replay config.
///
/// Returns `(ticks, times)` with `steps + 1` points; index 0 is the initial price.
pub trait SdeStrategy: Send + Sync {
    fn ticks(
        &self,
        initial_price: f64,
        time_horizon: f64,
        steps: u32,
        seed: u64,
    ) -> (Vec<f64>, Vec<f64>);
}

/// Geometric Brownian Motion strategy (no extra parameters).
pub struct GbmStrategy {
    pub drift: f64,
    pub volatility: f64,
}

/// Merton jump-diffusion strategy.
///
/// `jump_volatility` is a volatility, converted to variance for RustQuant.
pub struct MertonStrategy {
    pub drift: f64,
    pub volatility: f64,
    pub jump_intensity: f64,
    pub jump_mean: f64,
    pub jump_volatility: f64,
}

/// Ornstein-Uhlenbeck strategy (mean-reverting).
///
/// Maps `reversion_level` to RustQuant's long-run mean and `mean_reversion`
/// to its speed parameter; base `drift` is intentionally unused.
pub struct OuStrategy {
    pub volatility: f64,
    pub mean_reversion: f64,
    pub reversion_level: f64,
}

/// Heston stochastic-volatility strategy.
///
/// RustQuant's Heston process is unimplemented upstream, so the price +
/// variance pair is integrated here with full-truncation Euler and
/// correlated Brownian motions.
pub struct HestonStrategy {
    pub drift: f64,
    pub initial_variance: f64,
    pub mean_reversion: f64,
    pub reversion_level: f64,
    pub vol_of_vol: f64,
    pub correlation: f64,
}

impl SdeStrategy for GbmStrategy {
    fn ticks(
        &self,
        initial_price: f64,
        time_horizon: f64,
        steps: u32,
        seed: u64,
    ) -> (Vec<f64>, Vec<f64>) {
        let gbm = GeometricBrownianMotion::new(self.drift, self.volatility);
        let out = gbm.seedable_euler_maruyama(
            initial_price,
            0.0,
            time_horizon,
            steps as usize,
            1,
            false,
            seed,
        );
        let path = out.paths.into_iter().next().unwrap_or_default();
        (path, out.times)
    }
}

impl SdeStrategy for MertonStrategy {
    fn ticks(
        &self,
        initial_price: f64,
        time_horizon: f64,
        steps: u32,
        seed: u64,
    ) -> (Vec<f64>, Vec<f64>) {
        let merton = MertonJumpDiffusion::new(
            self.drift,
            self.volatility,
            self.jump_intensity,
            self.jump_mean,
            self.jump_volatility.powi(2),
        );
        let out = merton.seedable_euler_maruyama(
            initial_price,
            0.0,
            time_horizon,
            steps as usize,
            1,
            false,
            seed,
        );
        let path = out.paths.into_iter().next().unwrap_or_default();
        (path, out.times)
    }
}

impl SdeStrategy for OuStrategy {
    fn ticks(
        &self,
        initial_price: f64,
        time_horizon: f64,
        steps: u32,
        seed: u64,
    ) -> (Vec<f64>, Vec<f64>) {
        let ou = OrnsteinUhlenbeck::new(self.reversion_level, self.volatility, self.mean_reversion);
        let out = ou.seedable_euler_maruyama(
            initial_price,
            0.0,
            time_horizon,
            steps as usize,
            1,
            false,
            seed,
        );
        let path = out.paths.into_iter().next().unwrap_or_default();
        (path, out.times)
    }
}

impl SdeStrategy for HestonStrategy {
    fn ticks(
        &self,
        initial_price: f64,
        time_horizon: f64,
        steps: u32,
        seed: u64,
    ) -> (Vec<f64>, Vec<f64>) {
        // RustQuant's Heston StochasticProcess impl is unimplemented upstream,
        // so integrate the price + variance pair manually (full-truncation Euler).
        let n = steps as usize;
        let dt = time_horizon / steps as f64;
        let sqrt_dt = dt.sqrt();
        let rho = self.correlation.clamp(-1.0, 1.0);
        let ortho = (1.0 - rho * rho).max(0.0).sqrt();

        let mut rng = StdRng::seed_from_u64(seed);
        let normal = Normal::new(0.0, 1.0).expect("unit normal");

        let mut prices = vec![0.0; n + 1];
        let mut times = vec![0.0; n + 1];
        prices[0] = initial_price;
        let mut variance = self.initial_variance.max(0.0);

        for i in 0..n {
            times[i + 1] = (i + 1) as f64 * dt;
            let z1: f64 = normal.sample(&mut rng);
            let z2: f64 = normal.sample(&mut rng);
            let dw_price = sqrt_dt * z1;
            let dw_var = sqrt_dt * (rho * z1 + ortho * z2);

            let v_pos = variance.max(0.0);
            variance += self.mean_reversion * (self.reversion_level - v_pos) * dt
                + self.vol_of_vol * v_pos.sqrt() * dw_var;
            prices[i + 1] =
                prices[i] + self.drift * prices[i] * dt + v_pos.sqrt() * prices[i] * dw_price;
        }

        (prices, times)
    }
}

/// Build the strategy matching a stored simulation's model + frozen parameters.
///
/// Falls back to GBM when the stored model/params pair is inconsistent,
/// so legacy rows still replay instead of failing.
pub fn strategy_for(
    model_type: ModelType,
    params: &SimulationParams,
) -> Box<dyn SdeStrategy> {
    match (model_type, params.extra_params) {
        (ModelType::Merton, ModelParams::Merton {
            jump_intensity,
            jump_mean,
            jump_volatility,
        }) => Box::new(MertonStrategy {
            drift: params.drift,
            volatility: params.volatility,
            jump_intensity,
            jump_mean,
            jump_volatility,
        }),
        (ModelType::Ou, ModelParams::Ou {
            mean_reversion,
            reversion_level,
        }) => Box::new(OuStrategy {
            volatility: params.volatility,
            mean_reversion,
            reversion_level,
        }),
        (ModelType::Heston, ModelParams::Heston {
            initial_variance,
            mean_reversion,
            reversion_level,
            vol_of_vol,
            correlation,
        }) => Box::new(HestonStrategy {
            drift: params.drift,
            initial_variance,
            mean_reversion,
            reversion_level,
            vol_of_vol,
            correlation,
        }),
        _ => Box::new(GbmStrategy {
            drift: params.drift,
            volatility: params.volatility,
        }),
    }
}

/// Deterministic replay: seed + parameters regenerate the exact same path.
///
/// The `i64` seed is cast to `u64` so negative seeds stay reproducible.
pub fn generate_ticks(sim: &Simulation) -> (Vec<f64>, Vec<f64>) {
    let strategy = strategy_for(sim.model_type, &sim.parameters);
    strategy.ticks(
        sim.parameters.initial_price,
        sim.time_horizon,
        sim.steps,
        sim.random_seed as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::simulation::SimulationParams;

    fn gbm_sim(seed: i64, steps: u32) -> Simulation {
        Simulation {
            id: uuid::Uuid::new_v4(),
            model_type: ModelType::Gbm,
            time_horizon: 1.0,
            steps,
            random_seed: seed,
            stock_id: None,
            stock: Default::default(),
            parameters: toasty::Json(SimulationParams {
                initial_price: 100.0,
                drift: 0.05,
                volatility: 0.2,
                extra_params: ModelParams::Gbm,
            }),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }

    #[test]
    fn same_seed_replays_same_path() {
        let sim = gbm_sim(42, 10);
        let (a_ticks, a_times) = generate_ticks(&sim);
        let (b_ticks, b_times) = generate_ticks(&sim);
        assert_eq!(a_ticks, b_ticks);
        assert_eq!(a_times, b_times);
        assert_eq!(a_ticks.len(), 11);
        assert_eq!(a_ticks[0], 100.0);
    }

    #[test]
    fn different_seeds_diverge() {
        let (a, _) = generate_ticks(&gbm_sim(1, 10));
        let (b, _) = generate_ticks(&gbm_sim(2, 10));
        assert_ne!(a, b);
    }

    #[test]
    fn heston_manual_scheme_is_seeded_and_sized() {
        let mut sim = gbm_sim(7, 5);
        sim.model_type = ModelType::Heston;
        sim.parameters.extra_params = ModelParams::Heston {
            initial_variance: 0.04,
            mean_reversion: 2.0,
            reversion_level: 0.04,
            vol_of_vol: 0.3,
            correlation: -0.7,
        };
        let (a, times) = generate_ticks(&sim);
        let (b, _) = generate_ticks(&sim);
        assert_eq!(a, b);
        assert_eq!(a.len(), 6);
        assert_eq!(times.len(), 6);
        assert_eq!(a[0], 100.0);
    }
}
