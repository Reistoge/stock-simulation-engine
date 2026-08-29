use dotenv::dotenv;
use std::env;
mod routes;

#[tokio::main]
async fn main() {
    println!("-- Stock simulation Engine --");
    dotenv().ok();

    let version = env::var("VERSION").expect("VERSION VAR should be set");
    println!("VERSION: {version}");
    // build our application with a single route
    let app = routes::build_routes();
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
