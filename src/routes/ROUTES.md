# Creating Routes
To create a new route on this project you have to do this procedures:
1. Create a new file in `src/routes/<route-name.rs>`
2. Inside the new file put your route logic code
### Example
```rust
use axum::{Router, routing::get};

pub fn init() -> Router {
    Router::new().route(
        "/stocks",
        get(|| async { "GET stocks route!" })
            .post(|| async { "POST stocks route !" })
            .delete(|| async { "DELETE stocks route !" })
            .patch(|| async { "PATCH stocks route !" })
            .put(|| async { "PUT stocks route !" }),
    )
}
```
3. Go to `src/routes/mod.rs` import the new file with `mod <route-name.rs>`
4. Merge the new routes inside the `build_routes()` function using the `merge()` and put `init()` function from the new route as parameter 
### Example
```rust
/// mod.rs
mod <route-name>;
pub fn build_routes() -> Router { 
    Router::new()
        // .... other routes
        .merge(<route-name>::init())
}
```

