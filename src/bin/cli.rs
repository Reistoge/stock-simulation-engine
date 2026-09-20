#![cfg_attr(coverage_nightly, coverage(off))]
use dotenv::dotenv;
use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    dotenv().ok();
    let url = std::env::var("DB_URL").unwrap_or_else(|_| "postgresql::memory:".to_string());
    let _db = toasty::Db::builder()
        .models(toasty::models!(stock_simulation_engine::*))
        .connect(&url)
        .await?;

    let cli = ToastyCli::with_config(_db, config);
    cli.parse_and_run().await?;

    Ok(())
}
