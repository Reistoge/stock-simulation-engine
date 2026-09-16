use toasty::embed_migrations;

pub mod schema;

pub async fn init() -> toasty::Result<toasty::Db> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql::memory:".to_string());
    let db = match toasty::Db::builder()
        // `models!` discovers every `#[derive(Model)]` in this crate, so you don't list
        // them by hand.
        .models(toasty::models!(crate::*)) // read all the models
        .connect(&url)
        .await
    {
        Ok(db) => {
            println!("Db module connected !");
            db
        }
        Err(err) => {
            println!("Error on Db module !");
            return Err(err.into());
        }
    };

    static MIGRATIONS: toasty::migration::MigrationSet = embed_migrations!();
    MIGRATIONS.apply(&db).await?;
    Ok(db)
}
