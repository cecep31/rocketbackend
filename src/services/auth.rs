use crate::auth::Claims;
use crate::config::JwtConfig;
use crate::entities::{sessions, users};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::{Rng, distributions::Alphanumeric};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserBrief {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserBrief,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Debug)]
pub enum AuthError {
    Db(DbErr),
    UserExists,
    InvalidCredentials,
    InvalidToken,
    Token(jsonwebtoken::errors::Error),
    Hash(bcrypt::BcryptError),
}

impl From<DbErr> for AuthError {
    fn from(err: DbErr) -> Self {
        Self::Db(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Token(err)
    }
}

impl From<bcrypt::BcryptError> for AuthError {
    fn from(err: bcrypt::BcryptError) -> Self {
        Self::Hash(err)
    }
}

fn generate_refresh_token() -> String {
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(86)
        .map(char::from)
        .collect();
    format!("pl_{}", random)
}

fn make_user_brief(user: &users::Model) -> UserBrief {
    UserBrief {
        id: user.id,
        email: user.email.clone(),
        username: user.username.clone(),
    }
}

async fn create_token_and_session(
    db: &DatabaseConnection,
    user: &users::Model,
    user_agent: Option<String>,
) -> Result<AuthTokenResponse, AuthError> {
    let now = Utc::now();
    let jwt = JwtConfig::get();
    let exp = now + Duration::hours(jwt.expiry_hours);
    let claims = Claims {
        user_id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        is_super_admin: user.is_super_admin,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt.secret.as_bytes()),
    )?;

    let refresh_token = generate_refresh_token();
    let session = sessions::ActiveModel {
        refresh_token: Set(refresh_token.clone()),
        user_id: Set(user.id),
        created_at: Set(Some(now.into())),
        user_agent: Set(user_agent),
        expires_at: Set(Some((now + Duration::days(30)).into())),
    };
    session.insert(db).await?;

    Ok(AuthTokenResponse {
        access_token,
        refresh_token,
        user: make_user_brief(user),
    })
}

pub async fn register(
    db: &DatabaseConnection,
    email: String,
    username: String,
    password: String,
) -> Result<RegisterResponse, AuthError> {
    let existing = users::Entity::find()
        .filter(
            users::Column::Email
                .eq(email.clone())
                .or(users::Column::Username.eq(username.clone())),
        )
        .one(db)
        .await?;

    if existing.is_some() {
        return Err(AuthError::UserExists);
    }

    let hashed = hash(password, DEFAULT_COST)?;
    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email),
        username: Set(Some(username)),
        password: Set(Some(hashed)),
        created_at: Set(Some(Utc::now().into())),
        updated_at: Set(Some(Utc::now().into())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(RegisterResponse {
        id: user.id,
        email: user.email,
        username: user.username,
    })
}

pub async fn login(
    db: &DatabaseConnection,
    identifier: &str,
    password: &str,
    user_agent: Option<String>,
) -> Result<AuthTokenResponse, AuthError> {
    let user = users::Entity::find()
        .filter(
            users::Column::Email
                .eq(identifier)
                .or(users::Column::Username.eq(identifier)),
        )
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    let Some(hashed_password) = &user.password else {
        return Err(AuthError::InvalidCredentials);
    };

    if !verify(password, hashed_password)? {
        return Err(AuthError::InvalidCredentials);
    }

    create_token_and_session(db, &user, user_agent).await
}

pub async fn check_username_availability(
    db: &DatabaseConnection,
    username: &str,
) -> Result<bool, AuthError> {
    let exists = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some();

    Ok(!exists)
}

pub async fn check_email_availability(
    db: &DatabaseConnection,
    email: &str,
) -> Result<bool, AuthError> {
    let exists = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some();

    Ok(!exists)
}

pub async fn refresh_token(
    db: &DatabaseConnection,
    refresh_token: &str,
    user_agent: Option<String>,
) -> Result<AuthTokenResponse, AuthError> {
    let session = sessions::Entity::find_by_id(refresh_token.to_string())
        .one(db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    if let Some(expires_at) = session.expires_at {
        if expires_at.with_timezone(&Utc) < Utc::now() {
            let _ = sessions::Entity::delete_by_id(refresh_token.to_string())
                .exec(db)
                .await;
            return Err(AuthError::InvalidToken);
        }
    }

    let user = users::Entity::find_by_id(session.user_id)
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    sessions::Entity::delete_by_id(refresh_token.to_string())
        .exec(db)
        .await?;
    create_token_and_session(db, &user, user_agent).await
}

pub async fn logout(db: &DatabaseConnection, refresh_token: &str) -> Result<(), AuthError> {
    sessions::Entity::delete_by_id(refresh_token.to_string())
        .exec(db)
        .await?;
    Ok(())
}
