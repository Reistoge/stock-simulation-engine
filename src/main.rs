use std::env;

use dotenv::dotenv;
use std::result::Result::Ok;
use stock_simulation_engine::{
    db, repositories::user::UserRepositoryImpl, routes, routes::AppState,
};

#[tokio::main]
async fn main() {
    println!("-- Stock simulation Engine --");
    dotenv().ok();
    
    let db = match db::init().await {
        Ok(db) => {
            println!("DB success");
            db
        }
        Err(e) => {
            eprintln!("Error connecting the db: \n{}", e);
            std::process::exit(1);
        }
    };

    let version = env::var("VERSION").expect("VERSION VAR should be set");
    println!("VERSION: {version}");
    // build our application with a single route, injecting the UserRepository
    let app_state = AppState {
        user_repository: UserRepositoryImpl::new(db),
    };
    let app = routes::build_routes(app_state);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
