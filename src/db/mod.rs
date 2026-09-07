use toasty::embed_migrations;

pub mod schema;

pub async fn init() -> toasty::Result<()> {
    let url = std::env::var("DB_URL").unwrap_or_else(|_| "postgresql::memory:".to_string());
    let _db = toasty::Db::builder()
        // `models!` discovers every `#[derive(Model)]` in this crate, so you don't list
        // them by hand.
        .models(toasty::models!(crate::*))// read all the models
        .connect(&url)
        .await
        .unwrap();

    static MIGRATIONS: toasty::migration::MigrationSet = embed_migrations!();
    MIGRATIONS.apply(&_db).await?;
    println!("Connected to postgresql\n");
    Ok(())
}
