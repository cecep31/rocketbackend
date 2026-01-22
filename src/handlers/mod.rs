mod health;
mod post;
mod tag;

use axum::Router;
use std::sync::Arc;
use tokio_postgres::Client;
use tower_http::cors::CorsLayer;

pub fn create_router() -> Router<Arc<Client>> {
    Router::new()
        .merge(health::routes())
        .merge(post::routes())
        .merge(tag::routes())
        .layer(CorsLayer::permissive())
}
