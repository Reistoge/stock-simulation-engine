# Plan: Typed `extra_params` + `simulations` history table

**Branch:** `feature/typed-extra-params-simulations`

## Goal

Stop storing 1,000-point result arrays. Persist only the simulation *parameters*
and a *seed*, and let the engine regenerate the exact same series on replay.

Two supporting decisions:

1. `extra_params` is a **tagged Rust enum** (`ModelParams`) stored as PostgreSQL
   `JSONB` — self-describing and compile-time type-safe per model.
2. A `simulations` table records each run (`time_horizon`, `steps`, `random_seed`)
   and **snapshots the full parameters** at run time, so editing the parent stock
   later cannot silently change what a past simulation replays.

## Data model (`src/db/schema/stock.rs`)

Three public types live alongside `Stock`:

### `ModelType` — discriminator column

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
         toasty::Embed, utoipa::ToSchema)]
#[column(type = text)]
#[serde(rename_all = "snake_case")]
pub enum ModelType { Gbm, Merton, Ou, Heston }
```

- Stored as `model_type TEXT` (string labels, **not** a native PG enum — avoids
  painful `ALTER TYPE` migrations), cheap to filter/index.
- Helper `ModelType::from_params(&ModelParams) -> ModelType` maps the JSON
  variant tag back to the column value and is used for create-time validation.

### `ModelParams` — the typed `extra_params`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum ModelParams {
    Gbm,
    Merton { jump_intensity: f64, jump_mean: f64, jump_volatility: f64 },
    Ou     { mean_reversion: f64, reversion_level: f64 },
    Heston { initial_variance: f64, mean_reversion: f64, reversion_level: f64,
             vol_of_vol: f64, correlation: f64 },
}
```

Serialized as e.g. `{"model":"merton","jump_intensity":5.0,"jump_mean":-0.1,...}`.
A GBM row cannot structurally hold Merton fields.

### `SimulationParams` — replays snapshot

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimulationParams {
    pub model_type: ModelType,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub extra_params: ModelParams,
}
```

Frozen into `simulations.parameters` (JSONB) at run time.

### `Stock` model changes

| Field | Type |
| --- | --- |
| `ticker` | `varchar(10)` (was `varchar(5)`) |
| `model_type` | `ModelType` (`text`) |
| `initial_price` | `f64` (`double precision`) |
| `drift` | `f64` |
| `volatility` | `f64` |
| `extra_params` | `toasty::Json<ModelParams>` (`jsonb`) |
| `simulations` | `has_many` → `Vec<Simulation>` |

### `Simulation` model (`src/db/schema/simulation.rs`)

| Field | Type |
| --- | --- |
| `id` | `uuid` pk |
| `stock_id` | `uuid`, indexed, `belongs_to stock` |
| `time_horizon` | `f64` (years) |
| `steps` | `u32` (ticks, e.g. 1000) |
| `random_seed` | `i64` (`bigint`) |
| `parameters` | `toasty::Json<SimulationParams>` (`jsonb`) |
| `created_at` / `updated_at` | timestamptz |

Registered in `src/db/schema/mod.rs` as `pub mod simulation;`.

## API (`src/routes/stocks/types.rs`)

`CreateStockPayload` now accepts `ticker`, `name`, `model_type`,
`initial_price`, `drift`, `volatility`, `extra_params` (reusing
`stock::ModelParams`, whose internal `model` tag drives deserialization).

`post_stocks` validates `ModelType::from_params(&extra_params) == model_type`
and returns `400` otherwise.

## Migration (`toasty/`)

`0001_stock_params_and_simulations.sql` (generated via Toasty's own
schema-diff machinery; the CLI's DB connect was not available in this
environment, so the identical library path was run offline):

```sql
ALTER TABLE "stocks" ALTER COLUMN "ticker" TYPE VARCHAR(10);
ALTER TABLE "stocks" ADD COLUMN "volatility" DOUBLE PRECISION NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "model_type" TEXT NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "drift" DOUBLE PRECISION NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "extra_params" JSONB NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "initial_price" DOUBLE PRECISION NOT NULL;
CREATE TABLE "simulations" (
    "id" UUID NOT NULL,
    "stock_id" UUID NOT NULL,
    "time_horizon" DOUBLE PRECISION NOT NULL,
    "steps" BIGINT NOT NULL,
    "random_seed" BIGINT NOT NULL,
    "parameters" JSONB NOT NULL,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "updated_at" TIMESTAMPTZ(6) NOT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "index_simulations_by_stock_id" ON "simulations" ("stock_id");
```

Committed alongside the updated `0001_snapshot.toml` and `history.toml`.

> Note: apply to a live DB with `cargo run --bin cli -- migration apply` once a
> PostgreSQL instance is up (not run here — Docker daemon unavailable).

## Replay contract

Engine input = `SimulationParams` (snapshot) + `time_horizon` + `steps` +
`random_seed`. Deterministic PRNG seeded by `random_seed` regenerates the exact
1,000-point array in milliseconds — no array storage.

## Verification

- `cargo build` — clean.
- `cargo test` — 4 passed (existing auth suite).