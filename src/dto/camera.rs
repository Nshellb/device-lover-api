use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::dto::Pagination;
use crate::models::CameraRow;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CameraResponse {
    pub id: Uuid,
    pub category: &'static str,
    pub slug: String,
    pub brand: String,
    pub brand_slug: String,
    pub name: String,
    pub series: String,
    /// First marketing month reported by the source, in YYYY-MM format.
    pub release_month: String,
    pub camera_type: String,
    pub sensor_format: String,
    pub effective_megapixels: f64,
    pub image_processor: String,
    pub lens_mount: String,
    pub max_continuous_fps: f64,
    pub continuous_shooting_note: Option<String>,
    pub video_spec: String,
    /// Body only, excluding the battery, memory card and lens.
    pub body_weight_g: i32,
    pub source_url: String,
    pub source_title: String,
    pub checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CameraRow> for CameraResponse {
    fn from(row: CameraRow) -> Self {
        Self {
            id: row.id,
            category: "camera",
            slug: row.slug,
            brand: row.brand,
            brand_slug: row.brand_slug,
            name: row.name,
            series: row.series,
            release_month: row.release_month,
            camera_type: row.camera_type,
            sensor_format: row.sensor_format,
            effective_megapixels: row.effective_megapixels,
            image_processor: row.image_processor,
            lens_mount: row.lens_mount,
            max_continuous_fps: row.max_continuous_fps,
            continuous_shooting_note: row.continuous_shooting_note,
            video_spec: row.video_spec,
            body_weight_g: row.body_weight_g,
            source_url: row.source_url,
            source_title: row.source_title,
            checked_at: row.checked_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CameraListResponse {
    pub items: Vec<CameraResponse>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CameraComparisonResponse {
    pub devices: Vec<CameraResponse>,
    pub canonical_path: String,
    pub schema_version: u8,
}
