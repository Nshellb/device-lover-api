use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;
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
pub struct ConfigurationRow {
    pub id: Uuid,
    pub device_id: Uuid,
    pub label: String,
    pub storage_gb: i32,
    pub ram_gb: Option<i32>,
    pub ram_status: String,
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
    pub status: String,
    pub raw_value: Option<Value>,
    pub display_value: String,
    pub detail: Option<String>,
    pub source_id: Option<Uuid>,
}
