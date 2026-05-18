use crate::auth::AdminUser;
use crate::database::DbPool;
use crate::error::AppError;
use crate::models::report::{
    EngagementMetricsResponse, OverviewStatsResponse, PostReportResponse, UserReportResponse,
};
use crate::response::ApiResponse;
use crate::services;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    start_date: Option<String>,
    end_date: Option<String>,
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    tag_id: Option<i32>,
}

#[derive(Serialize)]
pub struct OverviewReport {
    overview: OverviewStatsResponse,
    engagement: EngagementMetricsResponse,
}

fn date_range(query: &ReportQuery) -> services::report::DateRange<'_> {
    services::report::DateRange {
        start_date: query.start_date.as_deref(),
        end_date: query.end_date.as_deref(),
    }
}

pub async fn get_overview(
    State(pool): State<DbPool>,
    _admin_user: AdminUser,
    Valid(query): Valid<Query<ReportQuery>>,
) -> Result<Json<ApiResponse<OverviewReport>>, AppError> {
    let overview = services::report::overview(&pool).await?;
    let engagement = services::report::engagement(&pool, date_range(&query)).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Overview report fetched successfully",
        OverviewReport {
            overview,
            engagement,
        },
    )))
}

pub async fn get_users(
    State(pool): State<DbPool>,
    _admin_user: AdminUser,
    Valid(query): Valid<Query<ReportQuery>>,
) -> Result<Json<ApiResponse<UserReportResponse>>, AppError> {
    let report =
        services::report::user_report(&pool, date_range(&query), query.limit.unwrap_or(10)).await?;
    Ok(Json(ApiResponse::success_with_message(
        "User report fetched successfully",
        report,
    )))
}

pub async fn get_posts(
    State(pool): State<DbPool>,
    _admin_user: AdminUser,
    Valid(query): Valid<Query<ReportQuery>>,
) -> Result<Json<ApiResponse<PostReportResponse>>, AppError> {
    let report = services::report::post_report(
        &pool,
        date_range(&query),
        query.limit.unwrap_or(10),
        query.tag_id,
    )
    .await?;
    Ok(Json(ApiResponse::success_with_message(
        "Post report fetched successfully",
        report,
    )))
}

pub async fn get_engagement(
    State(pool): State<DbPool>,
    _admin_user: AdminUser,
    Valid(query): Valid<Query<ReportQuery>>,
) -> Result<Json<ApiResponse<EngagementMetricsResponse>>, AppError> {
    let report = services::report::engagement(&pool, date_range(&query)).await?;
    Ok(Json(ApiResponse::success_with_message(
        "Engagement metrics fetched successfully",
        report,
    )))
}

pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/api/reports/overview", get(get_overview))
        .route("/api/reports/users", get(get_users))
        .route("/api/reports/posts", get(get_posts))
        .route("/api/reports/engagement", get(get_engagement))
}
