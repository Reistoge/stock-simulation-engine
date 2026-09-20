# Working with the Repository layer

Every query to the database goes through a **repository**. A repository is a `#[async_trait]` that owns the `toasty::Db` handle and exposes typed, named methods for the entity it represents, so route controllers and logic files (`auth/`) never call the ORM directly. The trait also makes repositories mockable with `mockall` in tests.

## How can I add a new Repository ?

1. Create a new file **<entity>.rs** inside this folder (src/repositories)
2. Define a trait (the contract) and a concrete struct that wraps the database handle. The methods take `&mut self` because toasty's `Executor` is implemented with `&mut self` receivers (see `db.rs` in the toasty crate).

```rust
// stock.rs
use async_trait::async_trait;
use crate::db::schema::stock::Stock;

// `#[automock]` is a no-op in production builds and generates `MockStockRepository`
// when running tests, so the pattern is directly unit-testable.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StockRepository: Send + Sync {
    async fn find_by_ticker(&mut self, ticker: &str) -> Result<Stock, toasty::Error>;
    async fn create(&mut self, name: &str, ticker: &str) -> Result<uuid::Uuid, toasty::Error>;
}

#[derive(Clone)] // cloneable so axum can hand a copy to each request
pub struct StockRepositoryImpl {
    db: toasty::Db,
}

#[async_trait]
impl StockRepository for StockRepositoryImpl {
    async fn find_by_ticker(&mut self, ticker: &str) -> Result<Stock, toasty::Error> {
        Stock::filter(Stock::fields().ticker().eq(ticker))
            .exec(&mut self.db)
            .await
    }

    async fn create(&mut self, name: &str, ticker: &str) -> Result<uuid::Uuid, toasty::Error> {
        Stock::create()
            .name(name)
            .ticker(ticker)
            .exec(&mut self.db)
            .await
            .map(|stock| stock.id)
    }
}
```

* **Note**: See the docs of toasty or the existing repository (`user.rs`) to see how to build queries, constraints and relationships.
    * [Defining Models](https://tokio-rs.github.io/toasty/nightly/guide/defining-models.html)
    * [Queries & Statements](https://tokio-rs.github.io/toasty/nightly/guide/queries.html)

3. Register the new repository in the src/repositories/mod.rs file
```rust
// mod.rs
pub mod user;        <-- existing repositories
pub mod stock;       <-- your new repository !
```
**Note**: This makes your repository visible to the rest of the code.

4. Add the concrete implementation as a field of the shared `AppState` in src/routes/mod.rs so it can be injected into handlers.
```rust
// routes/mod.rs
#[derive(Clone)]
pub struct AppState {
    pub user_repository: user::UserRepositoryImpl,         <-- existing
    pub stock_repository: repositories::stock::StockRepositoryImpl, <-- your new repository !
}
```

5. Inject it instead of the `Db`: extract `AppState` in the handler and pass the repository down.
```rust
// routes/stocks/mod.rs
use crate::routes::AppState;

#[utoipa::path(...)]
async fn post_stocks(
    State(mut app): State<AppState>, // <-- the repository, not `toasty::Db`
    Json(payload): Json<CreateStockPayload>,
) -> ... {
    let repo = &mut app.stock_repository;
    let stock_id = repo.create(&payload.ticker, &payload.name).await...;
}
```

**Note**: Repositories never touch passwords, JWT or validation; that logic stays in the logic layer (`auth/`). Repositories are only concerned with reading and writing the database.

# Workflow — consuming a repository
1. Define the query as a named method on the repository trait (`find_by_email`, `create`, ...)
2. In the handler, extract `State(mut app): State<AppState>` and call `app.<entity>_repository.<method>(...)`
3. Map the `Result` to the proper HTTP status with `match` (see the error handling convention in the README)

# Testing with mockall
1. Add `mockall` under `[dev-dependencies]` in Cargo.toml
2. `#[cfg_attr(test, mockall::automock)]` on the trait generates `Mock<Entity>Repository` in tests only
3. Handlers accept `&mut dyn <Entity>Repository`, so tests pass a mock and set expectations:

```rust
#[tokio::test]
async fn register_creates_user() {
    let mut repo = MockUserRepository::new();
    repo.expect_email_exists()
        .with(eq("ada@example.com"))
        .times(1)
        .returning(|_| false);
    repo.expect_create()
        .with(eq("ada"), eq("ada@example.com"), always())
        .times(1)
        .returning(|_, _, _| Ok(Uuid::new_v4()));

    let response = register_handler(&mut repo, Json(register_info)).await;
    assert!(response.is_ok());
}
```
See `src/auth/validation.rs` for the full registration test suite.