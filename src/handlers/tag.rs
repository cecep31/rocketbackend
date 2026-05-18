use crate::database::DbPool;
use crate::error::AppError;
use crate::models::tag::{SitemapTag, Tag};
use crate::response::ApiResponse;
use crate::services;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use axum_valid::Valid;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TagPaginationQuery {
    #[validate(range(min = 0, max = 10_000))]
    offset: Option<i64>,
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
}

pub async fn get_tags(
    State(pool): State<DbPool>,
    Valid(query): Valid<Query<TagPaginationQuery>>,
) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    let client = pool;
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    let (tags, total) = services::tag::get_all_tags(&client, offset, limit).await?;
    Ok(Json(ApiResponse::with_meta_message(
        "Successfully retrieved tags",
        tags,
        total,
        limit,
        offset,
    )))
}

pub async fn get_tags_for_sitemap(
    State(pool): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<SitemapTag>>>, AppError> {
    let client = pool;
    let tags = services::tag::get_tags_for_sitemap(&client, 1000).await?;
    Ok(Json(ApiResponse::success_with_message(
        "Successfully retrieved tags for sitemap",
        tags,
    )))
}

#[derive(Deserialize, Validate)]
pub struct TagIdPath {
    #[validate(range(min = 1))]
    id: i32,
}

pub async fn get_tag_by_id(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<TagIdPath>>,
) -> Result<Json<ApiResponse<Tag>>, AppError> {
    let client = pool;
    match services::tag::get_tag_by_id(&client, params.id).await {
        Ok(Some(tag)) => Ok(Json(ApiResponse::success_with_message(
            "Successfully retrieved tag",
            tag,
        ))),
        Ok(None) => Err(AppError::NotFound("Tag not found".to_string())),
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/api/tags", get(get_tags))
        .route("/api/tags/sitemap", get(get_tags_for_sitemap))
        .route("/api/tags/{id}", get(get_tag_by_id))
}
