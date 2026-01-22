use crate::models::response::ApiResponse;
use crate::models::tag::Tag;
use crate::services;
use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;
use tokio_postgres::Client;

pub async fn get_tags(State(conn): State<Arc<Client>>) -> Json<ApiResponse<Vec<Tag>>> {
    let tags = services::tag::get_all_tags(&conn).await.unwrap_or_else(|_| vec![]);
    let total = tags.len() as i64;
    Json(ApiResponse::with_meta(tags, total, None, None))
}

pub fn routes() -> Router<Arc<Client>> {
    Router::new().route("/v1/tags", get(get_tags))
}
