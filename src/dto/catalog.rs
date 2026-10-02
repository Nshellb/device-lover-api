use std::collections::BTreeMap;
use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
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
    pub image_alt: Option<String>,
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
    /// Up to 3 entries (e.g. 펼친 상태 / 접은 상태), each with separate numeric
    /// width/height/depth in millimetres so they can be searched individually.
    pub dimensions: Vec<DeviceDimension>,
    pub materials: Vec<DeviceMaterial>,
    pub power: DevicePower,
    /// Operating system / UX versions: the launch version plus upgrade targets.
    pub software: Vec<DeviceSoftware>,
    pub colors: Vec<DeviceColor>,
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
pub struct DeviceDimension {
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DimensionInput {
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceMaterial {
    pub part: String,
    pub material: String,
    pub note: Option<String>,
}

pub type MaterialInput = DeviceMaterial;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSoftware {
    pub version_id: Uuid,
    /// "os" or "ux"
    pub category: String,
    pub value: String,
    pub label: String,
    /// True for the version the device launched with; false for upgrade targets.
    pub is_launch: bool,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInput {
    pub version_id: Uuid,
    pub is_launch: bool,
    pub note: Option<String>,
}

/// Battery and charging as numbers. `null` = unknown; charging `0` = not supported.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DevicePower {
    pub battery_mah: Option<i32>,
    pub battery_note: Option<String>,
    pub wired_w: Option<f64>,
    pub wired_note: Option<String>,
    pub wireless_w: Option<f64>,
    pub wireless_note: Option<String>,
}

pub type PowerInput = DevicePower;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfiguration {
    pub id: Uuid,
    pub label: String,
    pub storage_gb: i32,
    pub ram_gb: Option<i32>,
    /// Launch price in KRW; null when unknown.
    pub price_krw: Option<i32>,
    /// Launch price in USD; null when unknown.
    pub price_usd: Option<f64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceColor {
    pub id: Uuid,
    pub name: String,
    pub image_url: Option<String>,
    pub color_code: Option<String>,
    pub exclusive: bool,
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
    pub value: String,
    pub detail: Option<String>,
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
    pub price_krw: Option<i32>,
    pub price_usd: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColorInput {
    pub name: String,
    pub image_url: Option<String>,
    pub color_code: Option<String>,
    pub exclusive: bool,
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
    #[serde(default)]
    pub image_alt: Option<String>,
    pub launch_video_url: Option<String>,
    pub publication_status: String,
    pub aliases: Vec<AliasInput>,
    pub sources: Vec<SourceInput>,
    pub configurations: Vec<ConfigurationInput>,
    pub dimensions: Vec<DimensionInput>,
    #[serde(default)]
    pub materials: Vec<MaterialInput>,
    pub power: PowerInput,
    #[serde(default)]
    pub software: Vec<SoftwareInput>,
    pub colors: Vec<ColorInput>,
    #[schema(value_type = Object)]
    pub specs: HashMap<String, SpecInput>,
}
