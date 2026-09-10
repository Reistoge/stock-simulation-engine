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