use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct DeviceRow {
    pub id: Uuid,
    pub slug: String,
    pub category: String,
    pub brand: String,
    pub brand_slug: String,
    pub name: String,
    pub release_date: NaiveDate,
    pub market_code: String,
    pub summary_variant_label: Option<String>,
    pub image_url: Option<String>,
    pub launch_video_url: Option<String>,
    pub publication_status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct AliasRow {
    pub device_id: Uuid,
    pub value: String,
    pub kind: String,
}

#[derive(sqlx::FromRow)]
pub struct DimensionRow {
    pub device_id: Uuid,
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    pub note: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct MaterialRow {
    pub device_id: Uuid,
    pub part: String,
    pub material: String,
    pub note: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct PowerRow {
    pub device_id: Uuid,
    pub battery_mah: Option<i32>,
    pub battery_note: Option<String>,
    pub wired_w: Option<f64>,
    pub wired_note: Option<String>,
    pub wireless_w: Option<f64>,
    pub wireless_note: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct ConfigurationRow {
    pub id: Uuid,
    pub device_id: Uuid,
    pub label: String,
    pub storage_gb: i32,
    pub ram_gb: Option<i32>,
    pub price_krw: Option<i32>,
    pub price_usd: Option<f64>,
}

#[derive(sqlx::FromRow)]
pub struct ColorRow {
    pub id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub image_url: Option<String>,
    pub color_code: Option<String>,
    pub exclusive: bool,
}

#[derive(sqlx::FromRow)]
pub struct SourceRow {
    pub id: Uuid,
    pub device_id: Uuid,
    pub url: String,
    pub title: String,
    pub checked_at: Option<DateTime<Utc>>,
    pub is_primary: bool,
}

#[derive(sqlx::FromRow)]
pub struct SpecRow {
    pub device_id: Uuid,
    pub spec_key: String,
    pub display_value: String,
    pub detail: Option<String>,
    pub source_id: Option<Uuid>,
}
