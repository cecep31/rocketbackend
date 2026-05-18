use crate::config::JwtConfig;
use crate::error::AppError;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub email: String,
    pub is_super_admin: Option<bool>,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub is_super_admin: bool,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get(axum::http::header::AUTHORIZATION) else {
            return Err((StatusCode::UNAUTHORIZED, "missing authorization header").into_response());
        };

        let Ok(header) = header.to_str() else {
            return Err((StatusCode::UNAUTHORIZED, "invalid authorization header").into_response());
        };

        let Some(token) = header.strip_prefix("Bearer ") else {
            return Err((StatusCode::UNAUTHORIZED, "invalid bearer token").into_response());
        };

        let token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JwtConfig::get().secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token").into_response())?;

        Ok(AuthUser {
            id: token.claims.user_id,
            email: token.claims.email,
            username: token.claims.username,
            is_super_admin: token.claims.is_super_admin.unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdminUser(pub AuthUser);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        if !auth_user.is_super_admin {
            return Err((StatusCode::FORBIDDEN, "admin access required").into_response());
        }
        Ok(AdminUser(auth_user))
    }
}

impl From<AuthUser> for AppError {
    fn from(_: AuthUser) -> Self {
        AppError::InternalServerError("unexpected auth conversion".to_string())
    }
}
