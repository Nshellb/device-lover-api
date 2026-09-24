use std::collections::{HashMap, HashSet};

use axum::extract::{RawQuery, State};
use axum::routing::get;
use axum::{Json, Router};
use unicode_normalization::UnicodeNormalization;

use crate::catalog::{MAX_COMPARISON_DEVICES, SCHEMA_VERSION, normalize_route_identifier};
use crate::dto::{CameraComparisonResponse, CameraListResponse, CameraResponse, Pagination};
use crate::error::{ApiResult, AppError};
use crate::models::CameraRow;
use crate::state::AppState;

const CAMERA_FILTER: &str = r#"
    FROM camera_models cm
    JOIN brands b ON b.id = cm.brand_id
    WHERE ($2::text IS NULL OR cm.series = $2)
      AND NOT EXISTS (
          SELECT 1 FROM unnest($1::text[]) AS search(term)
          WHERE position(search.term IN lower(concat_ws(' ',
              b.name, b.slug, cm.name, cm.slug,
              CASE WHEN b.slug = 'canon' THEN '캐논 Canon' ELSE '' END
          ))) = 0
      )
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/cameras", get(list_cameras))
        .route("/api/v1/cameras/comparisons", get(compare_cameras))
        .route("/api/v1/admin/cameras", get(list_admin_cameras))
}

#[utoipa::path(
    get,
    path = "/api/v1/cameras",
    tag = "catalog",
    params(
        ("q" = Option<String>, Query, description = "Model or brand search, up to 100 characters"),
        ("series" = Option<String>, Query, description = "EOS 5D, EOS 6D or EOS x0D"),
        ("page" = Option<u32>, Query, description = "1–10000; defaults to 1"),
        ("page_size" = Option<u32>, Query, description = "1–100; defaults to 30")
    ),
    responses(
        (status = 200, body = CameraListResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_cameras(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<CameraListResponse>> {
    load_camera_list(&state, raw_query.as_deref())
        .await
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/cameras",
    tag = "admin",
    params(
        ("q" = Option<String>, Query, description = "Model or brand search, up to 100 characters"),
        ("series" = Option<String>, Query, description = "EOS 5D, EOS 6D or EOS x0D"),
        ("page" = Option<u32>, Query, description = "1–10000; defaults to 1"),
        ("page_size" = Option<u32>, Query, description = "1–100; defaults to 30")
    ),
    responses(
        (status = 200, body = CameraListResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_admin_cameras(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<CameraListResponse>> {
    load_camera_list(&state, raw_query.as_deref())
        .await
        .map(Json)
}

async fn load_camera_list(
    state: &AppState,
    raw_query: Option<&str>,
) -> ApiResult<CameraListResponse> {
    let query = CameraListQuery::parse(raw_query)?;
    let mut transaction = state.db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) {CAMERA_FILTER}"))
        .bind(&query.search_terms)
        .bind(&query.series)
        .fetch_one(&mut *transaction)
        .await?;
    let rows = sqlx::query_as::<_, CameraRow>(&format!(
        r#"
        SELECT cm.id, cm.slug, b.name AS brand, b.slug AS brand_slug,
               cm.name, cm.series, cm.release_month, cm.camera_type,
               cm.sensor_format, cm.effective_megapixels, cm.image_processor,
               cm.lens_mount, cm.max_continuous_fps, cm.continuous_shooting_note, cm.video_spec,
               cm.body_weight_g, cm.source_url, cm.source_title, cm.checked_at,
               cm.created_at, cm.updated_at
        {CAMERA_FILTER}
        ORDER BY cm.release_month DESC, cm.name COLLATE "C", cm.slug
        LIMIT $3 OFFSET $4
        "#
    ))
    .bind(&query.search_terms)
    .bind(&query.series)
    .bind(i64::from(query.page_size))
    .bind(i64::from((query.page - 1) * query.page_size))
    .fetch_all(&mut *transaction)
    .await?;
    transaction.commit().await?;

    let total = u64::try_from(total).map_err(|_| AppError::Internal)?;
    Ok(CameraListResponse {
        items: rows.into_iter().map(CameraResponse::from).collect(),
        pagination: Pagination {
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages: total.div_ceil(u64::from(query.page_size)),
        },
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/cameras/comparisons",
    tag = "catalog",
    params(("identifiers" = Vec<String>, Query, max_items = 3, min_items = 1)),
    responses(
        (status = 200, body = CameraComparisonResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn compare_cameras(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<CameraComparisonResponse>> {
    let identifiers = parse_comparison_query(raw_query.as_deref())?;
    let slugs = identifiers
        .iter()
        .map(|identifier| normalize_route_identifier(identifier))
        .collect::<Result<Vec<_>, _>>()?;
    if slugs.iter().collect::<HashSet<_>>().len() != slugs.len() {
        return Err(AppError::Validation(
            "identifiers must resolve to distinct cameras".into(),
        ));
    }

    let rows = sqlx::query_as::<_, CameraRow>(
        r#"
        SELECT cm.id, cm.slug, b.name AS brand, b.slug AS brand_slug,
               cm.name, cm.series, cm.release_month, cm.camera_type,
               cm.sensor_format, cm.effective_megapixels, cm.image_processor,
               cm.lens_mount, cm.max_continuous_fps, cm.continuous_shooting_note, cm.video_spec,
               cm.body_weight_g, cm.source_url, cm.source_title, cm.checked_at,
               cm.created_at, cm.updated_at
          FROM camera_models cm
          JOIN brands b ON b.id = cm.brand_id
         WHERE cm.slug = ANY($1)
        "#,
    )
    .bind(&slugs)
    .fetch_all(&state.db)
    .await?;
    let mut by_slug = rows
        .into_iter()
        .map(|row| (row.slug.clone(), row))
        .collect::<HashMap<_, _>>();
    if by_slug.len() != slugs.len() {
        return Err(AppError::NotFound);
    }
    let devices = slugs
        .iter()
        .map(|slug| {
            by_slug
                .remove(slug)
                .map(CameraResponse::from)
                .ok_or(AppError::NotFound)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let canonical_path = format!(
        "/{}",
        devices
            .iter()
            .map(|camera| camera.slug.as_str())
            .collect::<Vec<_>>()
            .join("-vs-")
    );

    Ok(Json(CameraComparisonResponse {
        devices,
        canonical_path,
        schema_version: SCHEMA_VERSION,
    }))
}

fn parse_comparison_query(raw_query: Option<&str>) -> ApiResult<Vec<String>> {
    let raw_query = raw_query.unwrap_or_default();
    let bytes = raw_query.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit())
        {
            return Err(AppError::Validation(
                "query contains invalid percent encoding".into(),
            ));
        }
    }
    let pairs = url::form_urlencoded::parse(bytes).collect::<Vec<_>>();
    if pairs.iter().any(|(key, _)| key != "identifiers") {
        return Err(AppError::Validation("unknown query parameter".into()));
    }
    let identifiers = pairs
        .into_iter()
        .map(|(_, value)| value.into_owned())
        .collect::<Vec<_>>();
    if !(1..=MAX_COMPARISON_DEVICES).contains(&identifiers.len()) {
        return Err(AppError::Validation(
            "identifiers must contain 1 to 3 values".into(),
        ));
    }
    Ok(identifiers)
}

struct CameraListQuery {
    search_terms: Vec<String>,
    series: Option<String>,
    page: u32,
    page_size: u32,
}

impl CameraListQuery {
    fn parse(raw_query: Option<&str>) -> ApiResult<Self> {
        let raw_query = raw_query.unwrap_or_default();
        let bytes = raw_query.as_bytes();
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'%'
                && (index + 2 >= bytes.len()
                    || !bytes[index + 1].is_ascii_hexdigit()
                    || !bytes[index + 2].is_ascii_hexdigit())
            {
                return Err(AppError::Validation(
                    "query contains invalid percent encoding".into(),
                ));
            }
        }

        let mut values = HashMap::<String, String>::new();
        for (key, value) in url::form_urlencoded::parse(bytes) {
            if !matches!(key.as_ref(), "q" | "series" | "page" | "page_size") {
                return Err(AppError::Validation(format!(
                    "unknown query parameter: {key}"
                )));
            }
            if values.insert(key.to_string(), value.into_owned()).is_some() {
                return Err(AppError::Validation(format!(
                    "query parameter must not be repeated: {key}"
                )));
            }
        }

        let q = values.remove("q").unwrap_or_default();
        if q.chars().count() > 100 {
            return Err(AppError::Validation(
                "q must be at most 100 characters".into(),
            ));
        }
        if q.chars().any(char::is_control) {
            return Err(AppError::Validation(
                "q must not contain control characters".into(),
            ));
        }
        let search_terms = q
            .nfkc()
            .flat_map(char::to_lowercase)
            .collect::<String>()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        let series = values.remove("series");
        if series
            .as_deref()
            .is_some_and(|value| !matches!(value, "EOS 5D" | "EOS 6D" | "EOS x0D"))
        {
            return Err(AppError::Validation(
                "series must be EOS 5D, EOS 6D or EOS x0D".into(),
            ));
        }
        let page = parse_page_value(values.remove("page"), "page", 10_000, 1)?;
        let page_size = parse_page_value(values.remove("page_size"), "page_size", 100, 30)?;

        Ok(Self {
            search_terms,
            series,
            page,
            page_size,
        })
    }
}

fn parse_page_value(
    value: Option<String>,
    name: &str,
    maximum: u32,
    default: u32,
) -> ApiResult<u32> {
    let Some(value) = value else {
        return Ok(default);
    };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AppError::Validation(format!("{name} must be an integer")));
    }
    let parsed = value
        .parse::<u32>()
        .map_err(|_| AppError::Validation(format!("{name} must be an integer")))?;
    if !(1..=maximum).contains(&parsed) {
        return Err(AppError::Validation(format!(
            "{name} must be between 1 and {maximum}"
        )));
    }
    Ok(parsed)
}
