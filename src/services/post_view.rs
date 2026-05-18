use crate::entities::{post_views, posts};
use crate::models::post_view::{PostViewResponse, PostViewStats, ViewStatusResponse};
use crate::models::user::UserResponse;
use crate::services::user_hydration;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub enum PostViewError {
    Db(DbErr),
    PostNotFound,
}

impl From<DbErr> for PostViewError {
    fn from(err: DbErr) -> Self {
        Self::Db(err)
    }
}

async fn post_exists(db: &DatabaseConnection, post_id: Uuid) -> Result<bool, DbErr> {
    Ok(posts::Entity::find_by_id(post_id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some())
}

async fn has_user_viewed(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<bool, DbErr> {
    Ok(post_views::Entity::find()
        .filter(post_views::Column::PostId.eq(post_id))
        .filter(post_views::Column::UserId.eq(user_id))
        .filter(post_views::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some())
}

fn hydrate_view(
    view: post_views::Model,
    users_by_id: &std::collections::HashMap<Uuid, UserResponse>,
) -> PostViewResponse {
    let user_response = view
        .user_id
        .and_then(|user_id| users_by_id.get(&user_id).cloned());
    PostViewResponse::from_entity(view, user_response)
}

async fn load_view_user_map(
    db: &DatabaseConnection,
    views: &[post_views::Model],
) -> Result<std::collections::HashMap<Uuid, UserResponse>, DbErr> {
    user_hydration::load_user_response_map(db, views.iter().filter_map(|view| view.user_id)).await
}

pub async fn record_view(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Option<Uuid>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<(), PostViewError> {
    if !post_exists(db, post_id).await? {
        return Err(PostViewError::PostNotFound);
    }

    if let Some(user_id) = user_id
        && has_user_viewed(db, post_id, user_id).await?
    {
        return Ok(());
    }

    let now = Utc::now();
    post_views::ActiveModel {
        id: Set(Uuid::new_v4()),
        post_id: Set(post_id),
        user_id: Set(user_id),
        ip_address: Set(ip_address.filter(|value| !value.trim().is_empty())),
        user_agent: Set(user_agent.filter(|value| !value.trim().is_empty())),
        created_at: Set(Some(now.into())),
        updated_at: Set(Some(now.into())),
        deleted_at: Set(None),
    }
    .insert(db)
    .await?;

    Ok(())
}

pub async fn get_views_by_post_id(
    db: &DatabaseConnection,
    post_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<PostViewResponse>, i64), PostViewError> {
    if !post_exists(db, post_id).await? {
        return Err(PostViewError::PostNotFound);
    }

    let query = post_views::Entity::find()
        .filter(post_views::Column::PostId.eq(post_id))
        .filter(post_views::Column::DeletedAt.is_null());

    let total = query.clone().count(db).await? as i64;
    let view_models = query
        .order_by_desc(post_views::Column::CreatedAt)
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    let users_by_id = load_view_user_map(db, &view_models).await?;
    let views = view_models
        .into_iter()
        .map(|view| hydrate_view(view, &users_by_id))
        .collect();

    Ok((views, total))
}

pub async fn get_view_stats(
    db: &DatabaseConnection,
    post_id: Uuid,
) -> Result<PostViewStats, PostViewError> {
    if !post_exists(db, post_id).await? {
        return Err(PostViewError::PostNotFound);
    }

    let views = post_views::Entity::find()
        .filter(post_views::Column::PostId.eq(post_id))
        .filter(post_views::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    let total_views = views.len() as i64;
    let authenticated_views = views.iter().filter(|view| view.user_id.is_some()).count() as i64;
    let anonymous_views = total_views - authenticated_views;
    let unique_views = views
        .iter()
        .filter_map(|view| view.user_id)
        .collect::<HashSet<_>>()
        .len() as i64;

    Ok(PostViewStats {
        post_id,
        total_views,
        unique_views,
        anonymous_views,
        authenticated_views,
    })
}

pub async fn has_user_viewed_post(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<ViewStatusResponse, PostViewError> {
    if !post_exists(db, post_id).await? {
        return Err(PostViewError::PostNotFound);
    }

    Ok(ViewStatusResponse {
        has_viewed: has_user_viewed(db, post_id, user_id).await?,
    })
}
