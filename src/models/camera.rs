use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct CameraRow {
    pub id: Uuid,
    pub slug: String,
    pub brand: String,
    pub brand_slug: String,
    pub name: String,
    pub series: String,
    pub release_month: String,
    pub camera_type: String,
    pub sensor_format: String,
    pub effective_megapixels: f64,
    pub image_processor: String,
    pub lens_mount: String,
    pub max_continuous_fps: f64,
    pub continuous_shooting_note: Option<String>,
    pub video_spec: String,
    pub body_weight_g: i32,
    pub source_url: String,
    pub source_title: String,
    pub checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
