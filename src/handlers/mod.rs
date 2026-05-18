mod auth;
mod bookmark;
mod health;
mod holding;
mod notification;
mod post;
mod report;
mod tag;
mod user;

pub use post::OrderDirection;

use crate::database::DbPool;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn create_router() -> Router<DbPool> {
    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(bookmark::routes())
        .merge(holding::routes())
        .merge(notification::routes())
        .merge(post::routes())
        .merge(report::routes())
        .merge(tag::routes())
        .merge(user::routes())
        // TraceLayer should be added early to trace all requests
        // It provides good defaults: logs method, uri, status, latency automatically
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
