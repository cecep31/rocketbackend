use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::Client;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub success: bool,
    pub message: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        success: true,
        message: String::from("helloword"),
    })
}

pub fn routes() -> Router<Arc<Client>> {
    Router::new()
        .route("/", get(health))
        .route("/v1/health", get(health))
}
