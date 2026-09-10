# Creating Routes

To create a new route on this project you have to do this procedures:

1. Create a new file in `src/routes/<route-name.rs>`
2. Inside the new file put your route logic code

### Example

```rust
#[derive(Deserialize, ToSchema)]
struct CreateStockPayload {
    ticker: String,
    price: f64,
}

#[derive(Deserialize, IntoParams)]
struct StockQueryFilters {
    market: Option<String>,
}
#[utoipa::path(
    get,
    path ="/{ticker}",
    tag = "stocks",
    params(
        ("ticker" = String, Path, description = "The stock ticker symbol"),
        StockQueryFilters, // Automatically mapped as Query parameters
        ("authorization" = String, Header, description = "Bearer token") // Document header
    ),
    responses(
        (status = 200, description = "List all stocks successfully")
    )
)]
async fn get_stocks(
    Path(ticker): Path<String>,
    Query(filters): Query<StockQueryFilters>,
    headers: HeaderMap,
) -> String {
    let auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    format!("Ticker: {}, Market: {:?}", ticker, filters.market)
}

#[utoipa::path(
    post,
    path = "",
    tag = "stocks",
    request_body = CreateStockPayload, // Document the JSON body
    responses(
        (status = 201, description = "Create a stock successfully")
    )
)]
async fn post_stocks(Json(payload): Json<CreateStockPayload>) -> String {
    format!("Created stock {} at ${}", payload.ticker, payload.price)
}

pub fn init() -> OpenApiRouter {
    // routes! macro handles multiple HTTP methods on the same path automatically
    OpenApiRouter::new().routes(routes!(get_stocks, post_stocks))
}


```

3. Go to `src/routes/mod.rs` import the new file with `mod <route-name.rs>`
4. Merge the new routes inside the `build_routes()` function using the `merge()` and put `init()` function from the new route as parameter

### Example

```rust
/// mod.rs
mod <route-name>;
pub fn build_routes() -> Router {

    // 1. Build the OpenApiRouter and nest your OpenAPI-documented modules
    let (router, api) = OpenApiRouter::with_openapi(swagger::ApiDoc::openapi())
        .nest("/stocks",stocks::init())
        .nest(<route-name>,stocks::init())
        .split_for_parts();

    // 2. The `router` is now a standard axum::Router. Merge undocumented routes and SwaggerUI.
    router
        .nest("/ws", websocket::init()) // Use .nest or .merge depending on your websocket router setup
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
}
```
