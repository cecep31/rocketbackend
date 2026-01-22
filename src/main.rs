mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    let db_conn = database::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    let app = handlers::create_router().with_state(Arc::new(db_conn));

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
