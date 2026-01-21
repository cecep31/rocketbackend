mod database;
mod models;
mod routes;
mod services;

use axum::{Router, routing::get};
use routes::health::health;
use routes::post::{get_posts, get_random_posts};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let db_conn = database::connect()
        .await
        .expect("failed to connect to database");

    let app = Router::new()
    .route("/", get(health))
        .route("/v1/health", get(health))
        .route("/v1/posts", get(get_posts))
        .route("/v1/posts/random", get(get_random_posts))
        .with_state(Arc::new(db_conn))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
