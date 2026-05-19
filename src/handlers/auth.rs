use crate::auth::AuthUser;
use crate::database::DbPool;
use crate::dto::auth::{
    AvailabilityResponse, CheckUsernameRequest, EmailPath, LoginRequest, LogoutRequest,
    RefreshTokenRequest, RegisterRequest,
};
use crate::error::AppError;
use crate::rate_limit::{RateLimiter, rate_limit};
use crate::response::ApiResponse;
use crate::services::{self, auth::AuthError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{get, post},
};
use axum_valid::Valid;
use std::time::Duration;

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn map_auth_error(message: &'static str, err: AuthError) -> AppError {
    match err {
        AuthError::UserExists => {
            AppError::BadRequest("Email or username already exists".to_string())
        }
        AuthError::InvalidCredentials => {
            AppError::Unauthorized("Invalid identifier or password".to_string())
        }
        AuthError::InvalidToken => AppError::Unauthorized("Invalid or expired token".to_string()),
        AuthError::Db(err) => AppError::from(err),
        AuthError::Token(err) => AppError::InternalServerError(format!("{}: {}", message, err)),
        AuthError::Hash(err) => AppError::InternalServerError(format!("{}: {}", message, err)),
    }
}

pub async fn register(
    State(pool): State<DbPool>,
    Valid(Json(req)): Valid<Json<RegisterRequest>>,
) -> Result<
    (
        StatusCode,
        Json<ApiResponse<services::auth::RegisterResponse>>,
    ),
    AppError,
> {
    let user = services::auth::register(&pool, req.email, req.username, req.password)
        .await
        .map_err(|err| map_auth_error("Registration failed", err))?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            "User registered successfully",
            user,
        )),
    ))
}

pub async fn login(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Valid(Json(req)): Valid<Json<LoginRequest>>,
) -> Result<Json<ApiResponse<services::auth::AuthTokenResponse>>, AppError> {
    let response =
        services::auth::login(&pool, &req.identifier, &req.password, user_agent(&headers))
            .await
            .map_err(|err| map_auth_error("Login failed", err))?;

    Ok(Json(ApiResponse::success_with_message(
        "Login successful",
        response,
    )))
}

pub async fn check_username(
    State(pool): State<DbPool>,
    Valid(Json(req)): Valid<Json<CheckUsernameRequest>>,
) -> Result<Json<ApiResponse<AvailabilityResponse>>, AppError> {
    let available = services::auth::check_username_availability(&pool, &req.username)
        .await
        .map_err(|err| map_auth_error("Failed to check username availability", err))?;

    Ok(Json(ApiResponse::success_with_message(
        "Username availability checked",
        AvailabilityResponse {
            username: Some(req.username),
            email: None,
            available,
        },
    )))
}

pub async fn check_email(
    State(pool): State<DbPool>,
    Valid(Path(params)): Valid<Path<EmailPath>>,
) -> Result<Json<ApiResponse<AvailabilityResponse>>, AppError> {
    let available = services::auth::check_email_availability(&pool, &params.email)
        .await
        .map_err(|err| map_auth_error("Failed to check email availability", err))?;

    Ok(Json(ApiResponse::success_with_message(
        "Email availability checked",
        AvailabilityResponse {
            username: None,
            email: Some(params.email),
            available,
        },
    )))
}

pub async fn refresh_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Valid(Json(req)): Valid<Json<RefreshTokenRequest>>,
) -> Result<Json<ApiResponse<services::auth::AuthTokenResponse>>, AppError> {
    let response = services::auth::refresh_token(&pool, &req.refresh_token, user_agent(&headers))
        .await
        .map_err(|err| map_auth_error("Failed to refresh token", err))?;

    Ok(Json(ApiResponse::success_with_message(
        "Token refreshed successfully",
        response,
    )))
}

pub async fn logout(
    State(pool): State<DbPool>,
    _auth_user: AuthUser,
    Valid(Json(req)): Valid<Json<LogoutRequest>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _ = services::auth::logout(&pool, &req.refresh_token).await;
    Ok(Json(ApiResponse::success_with_message(
        "Logout successful",
        serde_json::Value::Null,
    )))
}

pub async fn profile(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<crate::models::user::UserResponse>>, AppError> {
    match services::user::get_by_id(&pool, auth_user.id).await {
        Ok(Some(user)) => Ok(Json(ApiResponse::success_with_message(
            "Profile retrieved successfully",
            user,
        ))),
        Ok(None) => Err(AppError::NotFound("User not found".to_string())),
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn routes() -> Router<DbPool> {
    let login_limiter = RateLimiter::new(5, Duration::from_secs(60));
    let register_limiter = RateLimiter::new(3, Duration::from_secs(60));
    let refresh_limiter = RateLimiter::new(20, Duration::from_secs(60));
    let availability_limiter = RateLimiter::new(30, Duration::from_secs(60));

    Router::new()
        .route(
            "/api/auth/register",
            post(register)
                .route_layer(middleware::from_fn_with_state(register_limiter, rate_limit)),
        )
        .route(
            "/api/auth/login",
            post(login).route_layer(middleware::from_fn_with_state(login_limiter, rate_limit)),
        )
        .route(
            "/api/auth/check-username",
            post(check_username).route_layer(middleware::from_fn_with_state(
                availability_limiter.clone(),
                rate_limit,
            )),
        )
        .route(
            "/api/auth/email/{email}",
            get(check_email).route_layer(middleware::from_fn_with_state(
                availability_limiter,
                rate_limit,
            )),
        )
        .route(
            "/api/auth/refresh",
            post(refresh_token)
                .route_layer(middleware::from_fn_with_state(refresh_limiter, rate_limit)),
        )
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/profile", get(profile))
}
