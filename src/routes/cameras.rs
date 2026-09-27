use std::collections::{HashMap, HashSet};

use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::{Postgres, Transaction};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::catalog::{
    MAX_COMPARISON_DEVICES, SCHEMA_VERSION, expand_search_synonyms, normalize_route_identifier,
};
use crate::dto::{
    CameraComparisonResponse, CameraListResponse, CameraResponse, CameraWriteRequest, Pagination,
};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::CameraRow;
use crate::routes::catalog::{map_write_db_error, upsert_brand, valid_slug};
use crate::state::AppState;

const CAMERA_FILTER: &str = r#"
    FROM camera_models cm
    JOIN brands b ON b.id = cm.brand_id
    WHERE ($2::text IS NULL OR cm.series = $2)
      AND NOT EXISTS (
          SELECT 1 FROM unnest($1::text[]) AS search(term)
          WHERE position(search.term IN lower(concat_ws(' ',
              b.name, b.slug, cm.name, cm.slug
          ))) = 0
      )
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/cameras", get(list_cameras).post(create_camera))
        .route("/api/v1/cameras/comparisons", get(compare_cameras))
        .route(
            "/api/v1/cameras/by-id/{id}",
            get(get_admin_camera_by_id).put(update_camera),
        )
        .route("/api/v1/admin/cameras", get(list_admin_cameras))
}

#[utoipa::path(
    get,
    path = "/api/v1/cameras",
    tag = "catalog",
    params(
        ("q" = Option<String>, Query, description = "Model or brand search, up to 100 characters"),
        ("series" = Option<String>, Query, description = "Camera series/product line label (free text, exact match)"),
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
        ("series" = Option<String>, Query, description = "Camera series/product line label (free text, exact match)"),
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

#[utoipa::path(
    post,
    path = "/api/v1/cameras",
    tag = "admin",
    request_body = CameraWriteRequest,
    responses(
        (status = 201, body = CameraResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_camera(
    State(state): State<AppState>,
    AppJson(payload): AppJson<CameraWriteRequest>,
) -> ApiResult<(StatusCode, Json<CameraResponse>)> {
    validate_camera_write_request(&payload)?;
    let mut transaction = state.db.begin().await?;
    let brand_id = upsert_brand(&mut transaction, &payload.brand_slug, &payload.brand_name).await?;

    let camera_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO camera_models
            (brand_id, slug, name, series, release_month, camera_type, sensor_format,
             effective_megapixels, image_processor, lens_mount, max_continuous_fps,
             continuous_shooting_note, video_spec, body_weight_g, source_url,
             source_title, checked_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING id
        "#,
    )
    .bind(brand_id)
    .bind(&payload.slug)
    .bind(&payload.name)
    .bind(&payload.series)
    .bind(&payload.release_month)
    .bind(&payload.camera_type)
    .bind(&payload.sensor_format)
    .bind(payload.effective_megapixels)
    .bind(&payload.image_processor)
    .bind(&payload.lens_mount)
    .bind(payload.max_continuous_fps)
    .bind(&payload.continuous_shooting_note)
    .bind(&payload.video_spec)
    .bind(payload.body_weight_g)
    .bind(&payload.source_url)
    .bind(&payload.source_title)
    .bind(payload.checked_at)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_write_db_error)?;

    let response = load_camera_by_id(&mut transaction, camera_id).await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cameras/by-id/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = CameraResponse),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_admin_camera_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CameraResponse>> {
    let mut transaction = state.db.begin().await?;
    let response = load_camera_by_id(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/cameras/by-id/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    request_body = CameraWriteRequest,
    responses(
        (status = 200, body = CameraResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_camera(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AppJson(payload): AppJson<CameraWriteRequest>,
) -> ApiResult<Json<CameraResponse>> {
    validate_camera_write_request(&payload)?;
    let mut transaction = state.db.begin().await?;
    let brand_id = upsert_brand(&mut transaction, &payload.brand_slug, &payload.brand_name).await?;

    let updated = sqlx::query(
        r#"
        UPDATE camera_models
           SET brand_id = $2, slug = $3, name = $4, series = $5, release_month = $6,
               camera_type = $7, sensor_format = $8, effective_megapixels = $9,
               image_processor = $10, lens_mount = $11, max_continuous_fps = $12,
               continuous_shooting_note = $13, video_spec = $14, body_weight_g = $15,
               source_url = $16, source_title = $17, checked_at = $18, updated_at = now()
         WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(brand_id)
    .bind(&payload.slug)
    .bind(&payload.name)
    .bind(&payload.series)
    .bind(&payload.release_month)
    .bind(&payload.camera_type)
    .bind(&payload.sensor_format)
    .bind(payload.effective_megapixels)
    .bind(&payload.image_processor)
    .bind(&payload.lens_mount)
    .bind(payload.max_continuous_fps)
    .bind(&payload.continuous_shooting_note)
    .bind(&payload.video_spec)
    .bind(payload.body_weight_g)
    .bind(&payload.source_url)
    .bind(&payload.source_title)
    .bind(payload.checked_at)
    .execute(&mut *transaction)
    .await
    .map_err(map_write_db_error)?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    let response = load_camera_by_id(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn load_camera_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> ApiResult<CameraResponse> {
    let row = sqlx::query_as::<_, CameraRow>(
        r#"
        SELECT cm.id, cm.slug, b.name AS brand, b.slug AS brand_slug,
               cm.name, cm.series, cm.release_month, cm.camera_type,
               cm.sensor_format, cm.effective_megapixels, cm.image_processor,
               cm.lens_mount, cm.max_continuous_fps, cm.continuous_shooting_note, cm.video_spec,
               cm.body_weight_g, cm.source_url, cm.source_title, cm.checked_at,
               cm.created_at, cm.updated_at
          FROM camera_models cm
          JOIN brands b ON b.id = cm.brand_id
         WHERE cm.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(CameraResponse::from(row))
}

fn validate_camera_write_request(payload: &CameraWriteRequest) -> ApiResult<()> {
    if !valid_slug(&payload.brand_slug, 80) {
        return Err(AppError::Validation(
            "brandSlug must be a lowercase slug".into(),
        ));
    }
    if payload.brand_name.trim().is_empty() {
        return Err(AppError::Validation("brandName must not be empty".into()));
    }
    if !valid_slug(&payload.slug, 120) {
        return Err(AppError::Validation("slug must be a lowercase slug".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    for (label, value) in [
        ("series", payload.series.as_str()),
        ("imageProcessor", payload.image_processor.as_str()),
        ("videoSpec", payload.video_spec.as_str()),
        ("sourceTitle", payload.source_title.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(AppError::Validation(format!("{label} must not be empty")));
        }
    }
    if !valid_release_month(&payload.release_month) {
        return Err(AppError::Validation(
            "releaseMonth must match YYYY-MM".into(),
        ));
    }
    if !matches!(payload.camera_type.as_str(), "DSLR" | "mirrorless" | "compact") {
        return Err(AppError::Validation(
            "cameraType must be DSLR, mirrorless, or compact".into(),
        ));
    }
    if !matches!(
        payload.sensor_format.as_str(),
        "full_frame" | "aps_c" | "micro_four_thirds" | "one_inch"
    ) {
        return Err(AppError::Validation(
            "sensorFormat must be full_frame, aps_c, micro_four_thirds, or one_inch".into(),
        ));
    }
    if payload
        .lens_mount
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(AppError::Validation(
            "lensMount must not be empty when present".into(),
        ));
    }
    if !(payload.effective_megapixels > 0.0 && payload.effective_megapixels.is_finite()) {
        return Err(AppError::Validation(
            "effectiveMegapixels must be a positive, finite number".into(),
        ));
    }
    if !(payload.max_continuous_fps > 0.0 && payload.max_continuous_fps.is_finite()) {
        return Err(AppError::Validation(
            "maxContinuousFps must be a positive, finite number".into(),
        ));
    }
    if payload.body_weight_g <= 0 {
        return Err(AppError::Validation("bodyWeightG must be positive".into()));
    }
    Ok(())
}

fn valid_release_month(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[0] != b'0'
        && bytes[4] == b'-'
        && matches!(
            &value[5..7],
            "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12"
        )
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
            .map(expand_search_synonyms)
            .collect();
        let series = values.remove("series");
        if let Some(value) = &series {
            if value.trim().is_empty()
                || value.chars().count() > 160
                || value.chars().any(char::is_control)
            {
                return Err(AppError::Validation(
                    "series must be 1 to 160 non-control characters".into(),
                ));
            }
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
