use crate::auth::AuthUser;
use crate::database::DbPool;
use crate::error::AppError;
use crate::models::post::{Post, SitemapPost};
use crate::response::ApiResponse;
use crate::services;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use axum_valid::Valid;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 7))]
    title: String,
    photo_url: Option<String>,
    #[validate(length(min = 7, max = 100), regex(path = *SLUG_RE))]
    slug: String,
    #[validate(length(min = 10))]
    body: String,
    #[serde(default)]
    published: bool,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1))]
    title: Option<String>,
    photo_url: Option<String>,
    #[validate(length(min = 1, max = 100), regex(path = *SLUG_RE))]
    slug: Option<String>,
    #[validate(length(min = 1))]
    body: Option<String>,
    published: Option<bool>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Validate)]
pub struct CommentRequest {
    #[validate(length(min = 1, max = 1000))]
    text: String,
}

#[derive(Deserialize, Validate)]
pub struct RandomPostQuery {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl From<OrderDirection> for services::post::SortDirection {
    fn from(value: OrderDirection) -> Self {
        match value {
            OrderDirection::Asc => Self::Asc,
            OrderDirection::Desc => Self::Desc,
        }
    }
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PaginationQuery {
    #[validate(range(min = 0, max = 10_000))]
    offset: Option<i64>,
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(max = 200))]
    search: Option<String>,
    #[serde(alias = "sort_by")]
    order_by: Option<String>,
    #[serde(alias = "sort_order")]
    order_direction: Option<OrderDirection>,
}

static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());
static SLUG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9-]+$").unwrap());
static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

pub async fn create_post(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Json(req)): Valid<Json<CreatePostRequest>>,
) -> Result<Json<ApiResponse<Post>>, AppError> {
    let post = services::post::create_post(
        &pool,
        services::post::CreatePostInput {
            title: req.title,
            photo_url: req.photo_url,
            slug: req.slug,
            body: req.body,
            published: req.published,
            tags: req.tags,
        },
        auth_user.id,
    )
    .await?;

    Ok(Json(ApiResponse::success_with_message(
        "Successfully created post",
        post,
    )))
}

fn map_comment_error(err: services::comment::CommentError) -> AppError {
    match err {
        services::comment::CommentError::Db(err) => AppError::from(err),
        services::comment::CommentError::PostNotFound => {
            AppError::NotFound("Post not found".to_string())
        }
        services::comment::CommentError::CommentNotFound => {
            AppError::NotFound("Comment not found".to_string())
        }
        services::comment::CommentError::NotOwner => {
            AppError::Forbidden("You are not the comment author".to_string())
        }
    }
}

fn map_post_view_error(err: services::post_view::PostViewError) -> AppError {
    match err {
        services::post_view::PostViewError::Db(err) => AppError::from(err),
        services::post_view::PostViewError::PostNotFound => {
            AppError::NotFound("Post not found".to_string())
        }
    }
}

fn map_post_like_error(err: services::post_like::PostLikeError) -> AppError {
    match err {
        services::post_like::PostLikeError::Db(err) => AppError::from(err),
        services::post_like::PostLikeError::PostNotFound => {
            AppError::NotFound("Post not found".to_string())
        }
        services::post_like::PostLikeError::AlreadyLiked => {
            AppError::BadRequest("You have already liked this post".to_string())
        }
        services::post_like::PostLikeError::NotLiked => {
            AppError::BadRequest("You have not liked this post".to_string())
        }
    }
}

async fn ensure_author(pool: &DbPool, post_id: Uuid, auth_user: &AuthUser) -> Result<(), AppError> {
    match services::post::is_author(pool, post_id, auth_user.id).await? {
        Some(true) => Ok(()),
        Some(false) => Err(AppError::Forbidden(
            "You are not the post author".to_string(),
        )),
        None => Err(AppError::NotFound("Post not found".to_string())),
    }
}

fn get_pagination_params(
    query: &PaginationQuery,
) -> (
    i64,
    i64,
    Option<&str>,
    Option<&str>,
    Option<services::post::SortDirection>,
) {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let search = query.search.as_deref();
    let order_by = query.order_by.as_deref();
    let order_direction = query.order_direction.map(Into::into);
    (offset, limit, search, order_by, order_direction)
}

pub async fn get_posts(
    State(pool): State<DbPool>,
    Valid(query): Valid<Query<PaginationQuery>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, AppError> {
    let client = pool;
    let (offset, limit, search, order_by, order_direction) = get_pagination_params(&query);

    let (posts, total) =
        services::post::get_all_posts(&client, offset, limit, search, order_by, order_direction)
            .await?;

    Ok(Json(ApiResponse::with_meta_message(
        "Successfully retrieved posts",
        posts,
        total,
        limit,
        offset,
    )))
}

pub async fn get_random_posts(
    State(pool): State<DbPool>,
    Valid(query): Valid<Query<RandomPostQuery>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, AppError> {
    let client = pool;
    let limit = query.limit.unwrap_or(6);
    let limit = limit.min(20);
    let posts = services::post::get_random_posts(&client, limit).await?;
    Ok(Json(ApiResponse::success_with_message(
        "Successfully retrieved posts",
        posts,
    )))
}

pub async fn get_trending_posts(
    State(pool): State<DbPool>,
    Valid(query): Valid<Query<RandomPostQuery>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, AppError> {
    let client = pool;
    let limit = query.limit.unwrap_or(10).min(100);
    let posts = services::post::get_trending_posts(&client, limit).await?;
    Ok(Json(ApiResponse::success_with_message(
        "Successfully retrieved trending posts",
        posts,
    )))
}

pub async fn get_posts_for_sitemap(
    State(pool): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<SitemapPost>>>, AppError> {
    let client = pool;
    let posts = services::post::get_posts_for_sitemap(&client, 1000).await?;
    Ok(Json(ApiResponse::success_with_message(
        "Successfully retrieved posts for sitemap",
        posts,
    )))
}

#[derive(Deserialize, Validate)]
pub struct TagPath {
    #[validate(length(min = 1, max = 50), regex(path = *TAG_RE))]
    pub tag: String,
}

pub async fn get_posts_by_tag(
    State(pool): State<DbPool>,
    Valid(Path(tag_path)): Valid<Path<TagPath>>,
    Valid(query): Valid<Query<PaginationQuery>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, AppError> {
    let client = pool;
    let (offset, limit, search, order_by, order_direction) = get_pagination_params(&query);

    let (posts, total) = services::post::get_posts_by_tag(
        &client,
        &tag_path.tag,
        offset,
        limit,
        search,
        order_by,
        order_direction,
    )
    .await?;

    Ok(Json(ApiResponse::with_meta_message(
        "Successfully retrieved posts by tag",
        posts,
        total,
        limit,
        offset,
    )))
}

fn header_string(headers: &axum::http::HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

pub async fn record_view(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    headers: axum::http::HeaderMap,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let ip_address =
        header_string(&headers, "x-forwarded-for").or_else(|| header_string(&headers, "x-real-ip"));
    let user_agent = header_string(&headers, "user-agent");

    services::post_view::record_view(&pool, params.id, Some(auth_user.id), ip_address, user_agent)
        .await
        .map_err(map_post_view_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "View recorded successfully",
        serde_json::Value::Null,
    )))
}

pub async fn get_post_views(
    State(pool): State<DbPool>,
    _auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
    Valid(query): Valid<Query<PaginationQuery>>,
) -> Result<Json<ApiResponse<Vec<crate::models::post_view::PostViewResponse>>>, AppError> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let (views, total) = services::post_view::get_views_by_post_id(&pool, params.id, limit, offset)
        .await
        .map_err(map_post_view_error)?;

    Ok(Json(ApiResponse::with_meta_message(
        "Successfully retrieved post views",
        views,
        total,
        limit,
        offset,
    )))
}

pub async fn get_post_view_stats(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<crate::models::post_view::PostViewStats>>, AppError> {
    let stats = services::post_view::get_view_stats(&pool, params.id)
        .await
        .map_err(map_post_view_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Successfully retrieved view statistics",
        stats,
    )))
}

pub async fn check_user_viewed(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<crate::models::post_view::ViewStatusResponse>>, AppError> {
    let status = services::post_view::has_user_viewed_post(&pool, params.id, auth_user.id)
        .await
        .map_err(map_post_view_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Successfully checked view status",
        status,
    )))
}

pub async fn like_post(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    services::post_like::like_post(&pool, params.id, auth_user.id)
        .await
        .map_err(map_post_like_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Post liked successfully",
        serde_json::Value::Null,
    )))
}

pub async fn unlike_post(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    services::post_like::unlike_post(&pool, params.id, auth_user.id)
        .await
        .map_err(map_post_like_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Post unliked successfully",
        serde_json::Value::Null,
    )))
}

pub async fn get_post_likes(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
    Valid(query): Valid<Query<PaginationQuery>>,
) -> Result<Json<ApiResponse<crate::models::post_like::PostLikeListResponse>>, AppError> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let likes = services::post_like::get_likes_by_post_id(&pool, params.id, limit, offset)
        .await
        .map_err(map_post_like_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Post likes retrieved successfully",
        likes,
    )))
}

pub async fn get_post_like_stats(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<crate::models::post_like::PostLikeStats>>, AppError> {
    let stats = services::post_like::get_like_stats(&pool, params.id)
        .await
        .map_err(map_post_like_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Like stats retrieved successfully",
        stats,
    )))
}

pub async fn check_user_liked(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<crate::models::post_like::LikeStatusResponse>>, AppError> {
    let status = services::post_like::has_user_liked_post(&pool, params.id, auth_user.id)
        .await
        .map_err(map_post_like_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Like status retrieved successfully",
        status,
    )))
}

pub async fn create_comment(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
    Valid(Json(req)): Valid<Json<CommentRequest>>,
) -> Result<
    (
        axum::http::StatusCode,
        Json<ApiResponse<crate::models::comment::CommentResponse>>,
    ),
    AppError,
> {
    let comment = services::comment::create_comment(&pool, params.id, req.text, auth_user.id)
        .await
        .map_err(map_comment_error)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            "Comment created successfully",
            comment,
        )),
    ))
}

pub async fn get_comments_by_post_id(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<Vec<crate::models::comment::CommentResponse>>>, AppError> {
    let comments = services::comment::get_comments_by_post_id(&pool, params.id)
        .await
        .map_err(map_comment_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Comments fetched successfully",
        comments,
    )))
}

pub async fn update_comment(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<CommentPath>>,
    Valid(Json(req)): Valid<Json<CommentRequest>>,
) -> Result<Json<ApiResponse<crate::models::comment::CommentResponse>>, AppError> {
    let _post_id = params.id;
    let comment =
        services::comment::update_comment(&pool, params.comment_id, req.text, auth_user.id)
            .await
            .map_err(map_comment_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Comment updated successfully",
        comment,
    )))
}

pub async fn delete_comment(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<CommentPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _post_id = params.id;
    services::comment::delete_comment(&pool, params.comment_id, auth_user.id)
        .await
        .map_err(map_comment_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Comment deleted successfully",
        serde_json::Value::Null,
    )))
}

#[derive(Deserialize, Validate)]
pub struct UsernamePath {
    #[validate(length(min = 1, max = 50), regex(path = *USERNAME_RE))]
    pub username: String,
}

pub async fn get_posts_by_username(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<UsernamePath>>,
    Valid(query): Valid<Query<PaginationQuery>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, AppError> {
    let client = pool;
    let (offset, limit, _, _, _) = get_pagination_params(&query);

    let (posts, total) =
        services::post::get_posts_by_username(&client, &params.username, offset, limit).await?;

    Ok(Json(ApiResponse::with_meta_message(
        "Successfully retrieved posts",
        posts,
        total,
        limit,
        offset,
    )))
}

#[derive(Deserialize, Validate)]
pub struct PostIdPath {
    pub id: uuid::Uuid,
}

#[derive(Deserialize, Validate)]
pub struct CommentPath {
    pub id: uuid::Uuid,
    pub comment_id: uuid::Uuid,
}

pub async fn get_post(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<Post>>, AppError> {
    let client = pool;
    match services::post::get_post_by_id(&client, params.id).await {
        Ok(Some(post)) => Ok(Json(ApiResponse::success_with_message(
            "Successfully retrieved post",
            post,
        ))),
        Ok(None) => Err(AppError::NotFound(format!("Post not found: {}", params.id))),
        Err(e) => Err(AppError::from(e)),
    }
}

pub async fn update_post(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
    Valid(Json(req)): Valid<Json<UpdatePostRequest>>,
) -> Result<Json<ApiResponse<Post>>, AppError> {
    ensure_author(&pool, params.id, &auth_user).await?;

    match services::post::update_post(
        &pool,
        params.id,
        services::post::UpdatePostInput {
            title: req.title,
            photo_url: req.photo_url,
            slug: req.slug,
            body: req.body,
            published: req.published,
            tags: req.tags,
        },
    )
    .await?
    {
        Some(post) => Ok(Json(ApiResponse::success_with_message(
            "Post updated successfully",
            post,
        ))),
        None => Err(AppError::NotFound("Post not found".to_string())),
    }
}

pub async fn delete_post(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<PostIdPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    ensure_author(&pool, params.id, &auth_user).await?;

    match services::post::soft_delete_post(&pool, params.id).await? {
        true => Ok(Json(ApiResponse::success_with_message(
            "Successfully deleted post",
            serde_json::Value::Null,
        ))),
        false => Err(AppError::NotFound("Post not found".to_string())),
    }
}

#[derive(Deserialize, Validate)]
pub struct PostPath {
    #[validate(length(min = 1, max = 50), regex(path = *USERNAME_RE))]
    pub username: String,
    #[validate(length(min = 1, max = 100), regex(path = *SLUG_RE))]
    pub slug: String,
}

pub async fn get_post_by_username_and_slug(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<PostPath>>,
) -> Result<Json<ApiResponse<Post>>, AppError> {
    let client = pool;
    match services::post::get_post_by_username_and_slug(&client, &params.username, &params.slug)
        .await
    {
        Ok(Some(post)) => Ok(Json(ApiResponse::success_with_message(
            "Successfully retrieved post",
            post,
        ))),
        Ok(None) => Err(AppError::NotFound(format!(
            "Post not found: {} by {}",
            params.slug, params.username
        ))),
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/api/posts", get(get_posts).post(create_post))
        .route("/api/posts/random", get(get_random_posts))
        .route("/api/posts/trending", get(get_trending_posts))
        .route("/api/posts/sitemap", get(get_posts_for_sitemap))
        .route("/api/posts/username/{username}", get(get_posts_by_username))
        .route(
            "/api/posts/u/{username}/{slug}",
            get(get_post_by_username_and_slug),
        )
        .route("/api/posts/tag/{tag}", get(get_posts_by_tag))
        .route(
            "/api/posts/{id}/comments",
            get(get_comments_by_post_id).post(create_comment),
        )
        .route(
            "/api/posts/{id}/comments/{comment_id}",
            put(update_comment).delete(delete_comment),
        )
        .route("/api/posts/{id}/view", post(record_view))
        .route("/api/posts/{id}/views", get(get_post_views))
        .route("/api/posts/{id}/view-stats", get(get_post_view_stats))
        .route("/api/posts/{id}/viewed", get(check_user_viewed))
        .route("/api/posts/{id}/like", post(like_post).delete(unlike_post))
        .route("/api/posts/{id}/likes", get(get_post_likes))
        .route("/api/posts/{id}/like-stats", get(get_post_like_stats))
        .route("/api/posts/{id}/liked", get(check_user_liked))
        .route(
            "/api/posts/{id}",
            get(get_post).put(update_post).delete(delete_post),
        )
}
