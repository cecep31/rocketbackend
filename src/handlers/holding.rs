use crate::auth::AuthUser;
use crate::database::DbPool;
use crate::error::AppError;
use crate::models::holding::{
    DuplicateResultItem, HoldingMonthComparisonResponse, HoldingMonthlyDataResponse,
    HoldingResponse, HoldingSummaryResponse, HoldingSyncResponse, HoldingTrendResponse,
    HoldingTypeResponse,
};
use crate::response::ApiResponse;
use crate::services::{self, holding::HoldingError};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use axum_valid::Valid;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct HoldingPath {
    id: i64,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HoldingQuery {
    #[validate(range(min = 1, max = 12))]
    month: Option<i32>,
    #[validate(range(min = 2000))]
    year: Option<i32>,
    sort_by: Option<String>,
    order: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct CreateHoldingRequest {
    #[validate(length(min = 1))]
    name: String,
    symbol: Option<String>,
    #[validate(length(min = 1))]
    platform: String,
    holding_type_id: i16,
    #[validate(length(equal = 3))]
    currency: String,
    invested_amount: String,
    current_value: String,
    units: Option<String>,
    avg_buy_price: Option<String>,
    current_price: Option<String>,
    last_updated: Option<String>,
    notes: Option<String>,
    #[validate(range(min = 1, max = 12))]
    month: i32,
    #[validate(range(min = 2000))]
    year: i32,
}

#[derive(Deserialize, Validate)]
pub struct UpdateHoldingRequest {
    #[validate(length(min = 1))]
    name: Option<String>,
    symbol: Option<String>,
    #[validate(length(min = 1))]
    platform: Option<String>,
    holding_type_id: Option<i16>,
    #[validate(length(equal = 3))]
    currency: Option<String>,
    invested_amount: Option<String>,
    current_value: Option<String>,
    units: Option<String>,
    avg_buy_price: Option<String>,
    current_price: Option<String>,
    last_updated: Option<String>,
    notes: Option<String>,
    #[validate(range(min = 1, max = 12))]
    month: Option<i32>,
    #[validate(range(min = 2000))]
    year: Option<i32>,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateHoldingRequest {
    #[validate(range(min = 1, max = 12))]
    from_month: i32,
    #[validate(range(min = 1900, max = 2100))]
    from_year: i32,
    #[validate(range(min = 1, max = 12))]
    to_month: i32,
    #[validate(range(min = 1900, max = 2100))]
    to_year: i32,
    #[serde(default)]
    overwrite: bool,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SummaryQuery {
    #[validate(range(min = 1, max = 12))]
    month: Option<i32>,
    #[validate(range(min = 2000))]
    year: Option<i32>,
}

#[derive(Deserialize, Validate)]
pub struct TrendsQuery {
    years: Option<String>,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CompareQuery {
    #[validate(range(min = 1, max = 12))]
    from_month: Option<i32>,
    from_year: Option<i32>,
    #[validate(range(min = 1, max = 12))]
    to_month: Option<i32>,
    to_year: Option<i32>,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyQuery {
    #[validate(range(min = 1, max = 12))]
    start_month: Option<i32>,
    start_year: Option<i32>,
    #[validate(range(min = 1, max = 12))]
    end_month: Option<i32>,
    end_year: Option<i32>,
}

fn map_holding_error(err: HoldingError) -> AppError {
    match err {
        HoldingError::Db(err) => AppError::from(err),
        HoldingError::NotFound => AppError::NotFound("Holding not found".to_string()),
        HoldingError::HoldingTypeNotFound => {
            AppError::BadRequest("Holding type not found".to_string())
        }
        HoldingError::InvalidDecimal(field) => {
            AppError::BadRequest(format!("Invalid decimal value for {}", field))
        }
        HoldingError::DuplicateSameMonth => {
            AppError::BadRequest("Cannot duplicate holdings into the same month".to_string())
        }
    }
}

fn parse_years(raw: Option<String>) -> Vec<i32> {
    raw.unwrap_or_default()
        .split(',')
        .filter_map(|value| value.trim().parse::<i32>().ok())
        .collect()
}

pub async fn get_holdings(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(query): Valid<Query<HoldingQuery>>,
) -> Result<Json<ApiResponse<Vec<HoldingResponse>>>, AppError> {
    let (current_month, current_year) = services::holding::default_current_month_year();
    let holdings = services::holding::get_holdings(
        &pool,
        auth_user.id,
        Some(query.month.unwrap_or(current_month)),
        Some(query.year.unwrap_or(current_year)),
        query.sort_by.as_deref(),
        query.order.as_deref(),
    )
    .await
    .map_err(map_holding_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Holdings fetched successfully",
        holdings,
    )))
}

pub async fn get_holding_by_id(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<HoldingPath>>,
) -> Result<Json<ApiResponse<HoldingResponse>>, AppError> {
    let holding = services::holding::get_holding_by_id(&pool, params.id, auth_user.id)
        .await
        .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holding fetched successfully",
        holding,
    )))
}

pub async fn create_holding(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Json(req)): Valid<Json<CreateHoldingRequest>>,
) -> Result<(axum::http::StatusCode, Json<ApiResponse<HoldingResponse>>), AppError> {
    let holding = services::holding::create_holding(
        &pool,
        auth_user.id,
        services::holding::CreateHoldingInput {
            name: req.name,
            symbol: req.symbol,
            platform: req.platform,
            holding_type_id: req.holding_type_id,
            currency: req.currency,
            invested_amount: req.invested_amount,
            current_value: req.current_value,
            units: req.units,
            avg_buy_price: req.avg_buy_price,
            current_price: req.current_price,
            last_updated: req.last_updated,
            notes: req.notes,
            month: req.month,
            year: req.year,
        },
    )
    .await
    .map_err(map_holding_error)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            "Holding created successfully",
            holding,
        )),
    ))
}

pub async fn update_holding(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<HoldingPath>>,
    Valid(Json(req)): Valid<Json<UpdateHoldingRequest>>,
) -> Result<Json<ApiResponse<HoldingResponse>>, AppError> {
    let holding = services::holding::update_holding(
        &pool,
        params.id,
        auth_user.id,
        services::holding::UpdateHoldingInput {
            name: req.name,
            symbol: req.symbol,
            platform: req.platform,
            holding_type_id: req.holding_type_id,
            currency: req.currency,
            invested_amount: req.invested_amount,
            current_value: req.current_value,
            units: req.units,
            avg_buy_price: req.avg_buy_price,
            current_price: req.current_price,
            last_updated: req.last_updated,
            notes: req.notes,
            month: req.month,
            year: req.year,
        },
    )
    .await
    .map_err(map_holding_error)?;

    Ok(Json(ApiResponse::success_with_message(
        "Holding updated successfully",
        holding,
    )))
}

pub async fn delete_holding(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Path(params)): Valid<Path<HoldingPath>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    services::holding::delete_holding(&pool, params.id, auth_user.id)
        .await
        .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holding deleted successfully",
        serde_json::Value::Null,
    )))
}

pub async fn get_holding_types(
    State(pool): State<DbPool>,
    _auth_user: AuthUser,
) -> Result<Json<ApiResponse<Vec<HoldingTypeResponse>>>, AppError> {
    let types = services::holding::get_holding_types(&pool)
        .await
        .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holding types fetched successfully",
        types,
    )))
}

pub async fn get_summary(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(query): Valid<Query<SummaryQuery>>,
) -> Result<Json<ApiResponse<HoldingSummaryResponse>>, AppError> {
    let summary = services::holding::summary(&pool, auth_user.id, query.month, query.year)
        .await
        .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holdings summary fetched successfully",
        summary,
    )))
}

pub async fn get_trends(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(query): Valid<Query<TrendsQuery>>,
) -> Result<Json<ApiResponse<Vec<HoldingTrendResponse>>>, AppError> {
    let trends = services::holding::trends(&pool, auth_user.id, parse_years(query.years.clone()))
        .await
        .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holdings trends fetched successfully",
        trends,
    )))
}

pub async fn compare_months(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(query): Valid<Query<CompareQuery>>,
) -> Result<Json<ApiResponse<HoldingMonthComparisonResponse>>, AppError> {
    let (current_month, current_year) = services::holding::default_current_month_year();
    let to_month = query.to_month.unwrap_or(current_month);
    let to_year = query.to_year.unwrap_or(current_year);
    let (default_from_month, default_from_year) = services::holding::prev_month(to_month, to_year);
    let from_month = query.from_month.unwrap_or(default_from_month);
    let from_year = query.from_year.unwrap_or(default_from_year);

    let result = services::holding::compare_months(
        &pool,
        auth_user.id,
        from_month,
        from_year,
        to_month,
        to_year,
    )
    .await
    .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Month comparison fetched successfully",
        result,
    )))
}

pub async fn get_monthly_data(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(query): Valid<Query<MonthlyQuery>>,
) -> Result<Json<ApiResponse<Vec<HoldingMonthlyDataResponse>>>, AppError> {
    let (current_month, current_year) = services::holding::default_current_month_year();
    let start_month = query.start_month.unwrap_or(current_month);
    let start_year = query.start_year.unwrap_or(current_year);
    let (default_end_month, default_end_year) =
        services::holding::prev_n_months(start_month, start_year, 11);
    let end_month = query.end_month.unwrap_or(default_end_month);
    let end_year = query.end_year.unwrap_or(default_end_year);

    let result = services::holding::monthly_data(
        &pool,
        auth_user.id,
        start_month,
        start_year,
        end_month,
        end_year,
    )
    .await
    .map_err(map_holding_error)?;
    Ok(Json(ApiResponse::success_with_message(
        "Holdings monthly data fetched successfully",
        result,
    )))
}

pub async fn sync_prices(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<HoldingSyncResponse>>, AppError> {
    let result = services::holding::sync_prices(&pool, auth_user.id).await;
    Ok(Json(ApiResponse::success_with_message(
        "Prices synced successfully for current month",
        result,
    )))
}

pub async fn duplicate_holdings(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Valid(Json(req)): Valid<Json<DuplicateHoldingRequest>>,
) -> Result<
    (
        axum::http::StatusCode,
        Json<ApiResponse<Vec<DuplicateResultItem>>>,
    ),
    AppError,
> {
    let result = services::holding::duplicate_holdings(
        &pool,
        auth_user.id,
        req.from_month,
        req.from_year,
        req.to_month,
        req.to_year,
        req.overwrite,
    )
    .await
    .map_err(map_holding_error)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            "Holdings duplicated successfully",
            result,
        )),
    ))
}

pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/api/holdings", get(get_holdings).post(create_holding))
        .route("/api/holdings/summary", get(get_summary))
        .route("/api/holdings/trends", get(get_trends))
        .route("/api/holdings/compare", get(compare_months))
        .route("/api/holdings/monthly", get(get_monthly_data))
        .route("/api/holdings/duplicate", post(duplicate_holdings))
        .route("/api/holdings/sync", post(sync_prices))
        .route(
            "/api/holdings/{id}",
            get(get_holding_by_id)
                .put(update_holding)
                .delete(delete_holding),
        )
        .route("/api/holding-types", get(get_holding_types))
}
