use crate::models::report::{EngagementMetricsResponse, OverviewStatsResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    pub tag_id: Option<i32>,
}

#[derive(Serialize)]
pub struct OverviewReport {
    pub overview: OverviewStatsResponse,
    pub engagement: EngagementMetricsResponse,
}

pub fn date_range(query: &ReportQuery) -> crate::services::report::DateRange<'_> {
    crate::services::report::DateRange {
        start_date: query.start_date.as_deref(),
        end_date: query.end_date.as_deref(),
    }
}
