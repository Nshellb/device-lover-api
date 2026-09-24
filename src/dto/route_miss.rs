use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteMissCreateRequest {
    pub requested_path: String,
    pub referrer: Option<String>,
    pub locale: Option<String>,
    pub viewport_class: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteMissSummary {
    pub requested_path: String,
    pub hits: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteMissSummaryResponse {
    pub items: Vec<RouteMissSummary>,
    pub window_days: u16,
}
