use std::time::Duration;

use sea_orm::{ConnectOptions};

pub async fn init() {
    let mut opt = ConnectOptions::new("postgres://postgres:pass@localhost:5432/rustock_db");
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .sqlx_logging_level(log::LevelFilter::Info);
    // .set_schema_search_path("my_schema"); // set default Postgres schema

    // let conn = Database::connect(opt).await.unwrap();
    
}
