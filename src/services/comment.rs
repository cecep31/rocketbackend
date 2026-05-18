use crate::entities::{post_comments, posts, profiles, users};
use crate::models::comment::CommentResponse;
use crate::models::user::UserResponse;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    ModelTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum CommentError {
    Db(DbErr),
    PostNotFound,
    CommentNotFound,
    NotOwner,
}

impl From<DbErr> for CommentError {
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

async fn hydrate_comment(
    db: &DatabaseConnection,
    comment: post_comments::Model,
) -> Result<CommentResponse, DbErr> {
    let user_response = match comment.clone().find_related(users::Entity).one(db).await? {
        Some(user) => {
            let profile = user.clone().find_related(profiles::Entity).one(db).await?;
            Some(UserResponse::from_entity(user, profile, None))
        }
        None => None,
    };
    Ok(CommentResponse::from_entity(comment, user_response))
}

pub async fn create_comment(
    db: &DatabaseConnection,
    post_id: Uuid,
    text: String,
    created_by: Uuid,
) -> Result<CommentResponse, CommentError> {
    if !post_exists(db, post_id).await? {
        return Err(CommentError::PostNotFound);
    }

    let now = Utc::now();
    let comment = post_comments::ActiveModel {
        id: Set(Uuid::new_v4()),
        post_id: Set(post_id),
        text: Set(text),
        created_by: Set(created_by),
        parent_comment_id: Set(None),
        created_at: Set(Some(now.into())),
        updated_at: Set(Some(now.into())),
        deleted_at: Set(None),
    }
    .insert(db)
    .await?;

    Ok(hydrate_comment(db, comment).await?)
}

pub async fn get_comments_by_post_id(
    db: &DatabaseConnection,
    post_id: Uuid,
) -> Result<Vec<CommentResponse>, CommentError> {
    if !post_exists(db, post_id).await? {
        return Err(CommentError::PostNotFound);
    }

    let comments = post_comments::Entity::find()
        .filter(post_comments::Column::PostId.eq(post_id))
        .filter(post_comments::Column::DeletedAt.is_null())
        .order_by_desc(post_comments::Column::CreatedAt)
        .all(db)
        .await?;

    let mut responses = Vec::with_capacity(comments.len());
    for comment in comments {
        responses.push(hydrate_comment(db, comment).await?);
    }
    Ok(responses)
}

pub async fn update_comment(
    db: &DatabaseConnection,
    comment_id: Uuid,
    text: String,
    user_id: Uuid,
) -> Result<CommentResponse, CommentError> {
    let Some(comment) = post_comments::Entity::find_by_id(comment_id)
        .filter(post_comments::Column::DeletedAt.is_null())
        .one(db)
        .await?
    else {
        return Err(CommentError::CommentNotFound);
    };

    if comment.created_by != user_id {
        return Err(CommentError::NotOwner);
    }

    let mut active = comment.into_active_model();
    active.text = Set(text);
    active.updated_at = Set(Some(Utc::now().into()));
    let updated = active.update(db).await?;

    Ok(hydrate_comment(db, updated).await?)
}

pub async fn delete_comment(
    db: &DatabaseConnection,
    comment_id: Uuid,
    user_id: Uuid,
) -> Result<(), CommentError> {
    let Some(comment) = post_comments::Entity::find_by_id(comment_id)
        .filter(post_comments::Column::DeletedAt.is_null())
        .one(db)
        .await?
    else {
        return Err(CommentError::CommentNotFound);
    };

    if comment.created_by != user_id {
        return Err(CommentError::NotOwner);
    }

    let mut active = comment.into_active_model();
    active.deleted_at = Set(Some(Utc::now().into()));
    active.updated_at = Set(Some(Utc::now().into()));
    active.update(db).await?;
    Ok(())
}
