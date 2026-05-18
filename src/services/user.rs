use crate::entities::{profiles, users};
use crate::models::user::UserResponse;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

async fn hydrate_user(db: &DatabaseConnection, user: users::Model) -> Result<UserResponse, DbErr> {
    let profile = user.clone().find_related(profiles::Entity).one(db).await?;
    Ok(UserResponse::from_entity(user, profile, None))
}

pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<UserResponse>, DbErr> {
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    match user {
        Some(user) => Ok(Some(hydrate_user(db, user).await?)),
        None => Ok(None),
    }
}

pub async fn get_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<UserResponse>, DbErr> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    match user {
        Some(user) => Ok(Some(hydrate_user(db, user).await?)),
        None => Ok(None),
    }
}

pub async fn get_users(
    db: &DatabaseConnection,
    offset: i64,
    limit: i64,
) -> Result<(Vec<UserResponse>, i64), DbErr> {
    let query = users::Entity::find().filter(users::Column::DeletedAt.is_null());
    let total = query.clone().count(db).await? as i64;
    let user_models = query
        .order_by_desc(users::Column::CreatedAt)
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    let mut responses = Vec::with_capacity(user_models.len());
    for user in user_models {
        responses.push(hydrate_user(db, user).await?);
    }

    Ok((responses, total))
}

pub async fn soft_delete(db: &DatabaseConnection, id: Uuid) -> Result<bool, DbErr> {
    let Some(user) = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?
    else {
        return Ok(false);
    };

    let mut active = user.into_active_model();
    active.deleted_at = Set(Some(Utc::now().into()));
    active.updated_at = Set(Some(Utc::now().into()));
    active.update(db).await?;

    Ok(true)
}
