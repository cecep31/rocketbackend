use crate::entities::{post_likes, posts, profiles, users};
use crate::models::post_like::{
    LikeStatusResponse, PostLikeListResponse, PostLikeResponse, PostLikeStats,
};
use crate::models::user::UserResponse;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum PostLikeError {
    Db(DbErr),
    PostNotFound,
    AlreadyLiked,
    NotLiked,
}

impl From<DbErr> for PostLikeError {
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

async fn like_exists(db: &DatabaseConnection, post_id: Uuid, user_id: Uuid) -> Result<bool, DbErr> {
    Ok(post_likes::Entity::find()
        .filter(post_likes::Column::PostId.eq(post_id))
        .filter(post_likes::Column::UserId.eq(user_id))
        .filter(post_likes::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some())
}

async fn hydrate_like(
    db: &DatabaseConnection,
    like: post_likes::Model,
) -> Result<PostLikeResponse, DbErr> {
    let user_response = match like.clone().find_related(users::Entity).one(db).await? {
        Some(user) => {
            let profile = user.clone().find_related(profiles::Entity).one(db).await?;
            Some(UserResponse::from_entity(user, profile, None))
        }
        None => None,
    };
    Ok(PostLikeResponse::from_entity(like, user_response))
}

pub async fn like_post(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<(), PostLikeError> {
    if !post_exists(db, post_id).await? {
        return Err(PostLikeError::PostNotFound);
    }
    if like_exists(db, post_id, user_id).await? {
        return Err(PostLikeError::AlreadyLiked);
    }

    let now = Utc::now();
    post_likes::ActiveModel {
        id: Set(Uuid::new_v4()),
        post_id: Set(post_id),
        user_id: Set(user_id),
        created_at: Set(Some(now.into())),
        updated_at: Set(Some(now.into())),
        deleted_at: Set(None),
    }
    .insert(db)
    .await?;

    Ok(())
}

pub async fn unlike_post(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<(), PostLikeError> {
    if !post_exists(db, post_id).await? {
        return Err(PostLikeError::PostNotFound);
    }

    let result = post_likes::Entity::delete_many()
        .filter(post_likes::Column::PostId.eq(post_id))
        .filter(post_likes::Column::UserId.eq(user_id))
        .filter(post_likes::Column::DeletedAt.is_null())
        .exec(db)
        .await?;

    if result.rows_affected == 0 {
        return Err(PostLikeError::NotLiked);
    }

    Ok(())
}

pub async fn get_likes_by_post_id(
    db: &DatabaseConnection,
    post_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<PostLikeListResponse, PostLikeError> {
    if !post_exists(db, post_id).await? {
        return Err(PostLikeError::PostNotFound);
    }

    let query = post_likes::Entity::find()
        .filter(post_likes::Column::PostId.eq(post_id))
        .filter(post_likes::Column::DeletedAt.is_null());

    let total = query.clone().count(db).await? as i64;
    let like_models = query
        .order_by_desc(post_likes::Column::CreatedAt)
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    let mut likes = Vec::with_capacity(like_models.len());
    for like in like_models {
        likes.push(hydrate_like(db, like).await?);
    }

    Ok(PostLikeListResponse {
        likes,
        total,
        limit,
        offset,
    })
}

pub async fn get_like_stats(
    db: &DatabaseConnection,
    post_id: Uuid,
) -> Result<PostLikeStats, PostLikeError> {
    if !post_exists(db, post_id).await? {
        return Err(PostLikeError::PostNotFound);
    }

    let total_likes = post_likes::Entity::find()
        .filter(post_likes::Column::PostId.eq(post_id))
        .filter(post_likes::Column::DeletedAt.is_null())
        .count(db)
        .await? as i64;

    Ok(PostLikeStats {
        post_id,
        total_likes,
    })
}

pub async fn has_user_liked_post(
    db: &DatabaseConnection,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<LikeStatusResponse, PostLikeError> {
    if !post_exists(db, post_id).await? {
        return Err(PostLikeError::PostNotFound);
    }

    Ok(LikeStatusResponse {
        has_liked: like_exists(db, post_id, user_id).await?,
        post_id,
        user_id,
    })
}
