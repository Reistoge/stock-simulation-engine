# Stock Simulation Engine

Rust API using axum and stochastic mathematician models to simulate stocks in realtime.

Implemented features
- [ ] API
    - [x] Axum template
    - [x] Routing
    - [ ] Data models
    - [X] User Authentication
    - [ ] Payload
- [ ] Websocket
    - [X] Ws template
    - [ ] Real time traffic data
- [ ] Engine
    - [ ] State behaviours
    - [ ] Core algorithms
- [ ] Pipeline
    - [ ] Github action
- [ ] Testing
- [X] Swagger


## Testing

### Running the tests

Unit tests are colocated with the code (`#[cfg(test)] mod tests`). The suite
currently covers the auth handlers in `src/auth/validation.rs` using mocked
repositories (`MockUserRepository`), so it needs no database or `.env` file.

```bash
cargo test
```

### Code coverage

Coverage uses `cargo-llvm-cov` and the nightly toolchain, because the
`coverage(off)` attribute used to exclude files is unstable.

```bash
cargo install cargo-llvm-cov
rustup toolchain install nightly
cargo +nightly llvm-cov
```

Files excluded from coverage:

- `db/` and `routes/` trees and `bin/cli.rs` via
  `#![cfg_attr(coverage_nightly, coverage(off))]` module attributes
  (feature gate lives in `src/lib.rs`)
- `cli` is also not built as a test target (`test = false` in `Cargo.toml`)

The exclusions only apply under the nightly toolchain: `coverage_nightly` is
set by `cargo-llvm-cov` only when compiling with nightly. On stable the
attributes are inert and the report includes every file.

### Running on a fresh machine

```bash
git clone <repo> && cd stock-simulation-engine
# install Rust stable (edition 2024 requires >= 1.85): https://rustup.rs
cargo test          # no DB or .env needed
```

To actually run the server (`cargo run`) you also need the `.env` variables
(`JWT_SECRET`, `VERSION`, `DATABASE_URL`) and a PostgreSQL instance, e.g.
`docker-compose up -d db`.

##  Error handling convention
To handle errors on this project we are gonna use the `match` operator
example:
``` rust
    let _db = match toasty::Db::builder()
        // `models!` discovers every `#[derive(Model)]` in this crate, so you don't list
        // them by hand.
        .models(toasty::models!(crate::*)) // read all the models
        .connect(&url)
        .await
    {
        Ok(db) => {
            println!("Db module connected !");
            db // return db value 
        }
        Err(err) => {
            println!("Error on Db module !");
            return Err(err.into()); // propagate the error
        }
    };
    Ok(_db) // return ok value
```