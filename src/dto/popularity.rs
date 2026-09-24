use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSelectionRequest {
    pub category: String,
    pub slug: String,
    pub action: String,
}

#[derive(Clone, Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PopularDeviceSummary {
    pub id: Uuid,
    pub slug: String,
    pub category: String,
    pub brand: String,
    pub brand_slug: String,
    pub name: String,
    pub release_date: NaiveDate,
    pub market_code: String,
    pub aliases: Vec<String>,
    pub model_numbers: Vec<String>,
    pub image_url: Option<String>,
    pub selection_count: u64,
}

#[derive(Clone, Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PopularDevicesResponse {
    pub items: Vec<PopularDeviceSummary>,
    pub window_days: u16,
    pub cache_ttl_seconds: u64,
}
