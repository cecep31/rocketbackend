use crate::models::post::Post;
use crate::models::response::ApiResponse;
use crate::services;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio_postgres::Client;

#[derive(Deserialize)]
pub struct RandomPostQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

pub async fn get_posts(
    State(conn): State<Arc<Client>>,
    query: Query<PaginationQuery>,
) -> Json<ApiResponse<Vec<Post>>> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    
    let (posts, total) = services::post::get_all_posts(&conn, offset, limit)
        .await
        .unwrap_or_else(|_| (vec![], 0));
    
    Json(ApiResponse::with_meta(posts, total, Some(limit), Some(offset)))
}

pub async fn get_random_posts(
    State(conn): State<Arc<Client>>,
    query: Query<RandomPostQuery>,
) -> Json<ApiResponse<Vec<Post>>> {
    let limit = query.limit.unwrap_or(6);
    let posts = services::post::get_random_posts(&conn, limit).await.unwrap_or_else(|_| vec![]);
    let total = posts.len() as i64;
    Json(ApiResponse::with_meta(posts, total, Some(limit), None))
}

pub fn routes() -> Router<Arc<Client>> {
    Router::new()
        .route("/v1/posts", get(get_posts))
        .route("/v1/posts/random", get(get_random_posts))
}
