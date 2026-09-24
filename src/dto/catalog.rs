use std::collections::BTreeMap;
use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSchemaResponse {
    pub schema_version: u8,
    pub category: &'static str,
    pub max_comparison_devices: u8,
    pub brands: Vec<CatalogBrand>,
    pub sections: Vec<SpecificationSection>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CatalogBrand {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SpecificationSection {
    pub key: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub rows: Vec<SpecificationRow>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SpecificationRow {
    pub key: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
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
    pub publication_status: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDetail {
    #[serde(flatten)]
    #[schema(inline)]
    pub summary: DeviceSummary,
    pub variant: Option<String>,
    pub configurations: Vec<DeviceConfiguration>,
    pub source_url: String,
    pub sources: Vec<DeviceSource>,
    #[schema(value_type = Object)]
    pub specs: BTreeMap<String, SpecValue>,
    pub updated_at: DateTime<Utc>,
    pub launch_video_url: Option<String>,
    /// Same values as `aliases`/`modelNumbers`, but with each one's `kind` —
    /// admin editing needs this to resubmit aliases with their original kind.
    pub alias_details: Vec<AliasDetail>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AliasDetail {
    pub value: String,
    pub kind: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfiguration {
    pub id: Uuid,
    pub label: String,
    pub storage_gb: i32,
    pub ram_gb: Option<i32>,
    pub ram_status: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSource {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub checked_at: Option<DateTime<Utc>>,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecValue {
    pub status: String,
    #[schema(value_type = Object, nullable = true)]
    pub raw: Option<Value>,
    pub value: String,
    pub detail: Option<String>,
    pub muted: bool,
    pub source_id: Option<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DeviceListResponse {
    pub items: Vec<DeviceSummary>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonResponse {
    pub devices: Vec<DeviceDetail>,
    pub canonical_path: String,
    pub schema_version: u8,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomeResponse {
    pub devices: Vec<DeviceDetail>,
    pub canonical_path: Option<String>,
    pub schema_version: u8,
    pub as_of: NaiveDate,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecInput {
    pub status: String,
    #[schema(value_type = Object, nullable = true)]
    pub raw: Option<Value>,
    pub value: String,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AliasInput {
    pub value: String,
    pub kind: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceInput {
    pub url: String,
    pub title: String,
    pub checked_at: Option<DateTime<Utc>>,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationInput {
    pub label: String,
    pub storage_gb: i32,
    pub ram_gb: Option<i32>,
    pub ram_status: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceWriteRequest {
    pub brand_slug: String,
    pub brand_name: String,
    pub slug: String,
    pub name: String,
    pub release_date: NaiveDate,
    pub variant: Option<String>,
    pub image_url: Option<String>,
    pub launch_video_url: Option<String>,
    pub publication_status: String,
    pub aliases: Vec<AliasInput>,
    pub sources: Vec<SourceInput>,
    pub configurations: Vec<ConfigurationInput>,
    #[schema(value_type = Object)]
    pub specs: HashMap<String, SpecInput>,
}
