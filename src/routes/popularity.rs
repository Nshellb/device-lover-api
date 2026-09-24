use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

use crate::dto::{DeviceSelectionRequest, PopularDeviceSummary, PopularDevicesResponse};
use crate::error::{ApiResult, AppError};
use crate::state::{AppState, CachedPopularDevices};

const POPULARITY_WINDOW_DAYS: u16 = 30;
const CACHE_TTL: Duration = Duration::from_secs(300);
const ITEMS_PER_CATEGORY: i64 = 5;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/device-selections", post(record_device_selection))
        .route("/api/v1/popular-devices", get(list_popular_devices))
}

#[utoipa::path(
    post,
    path = "/api/v1/device-selections",
    tag = "analytics",
    request_body = DeviceSelectionRequest,
    responses(
        (status = 204, description = "Selection recorded"),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn record_device_selection(
    State(state): State<AppState>,
    Json(request): Json<DeviceSelectionRequest>,
) -> ApiResult<StatusCode> {
    if !matches!(request.category.as_str(), "smartphone" | "camera") {
        return Err(AppError::Validation(
            "category must be smartphone or camera".into(),
        ));
    }
    if !matches!(request.action.as_str(), "view" | "compare") {
        return Err(AppError::Validation(
            "action must be view or compare".into(),
        ));
    }
    if !valid_slug(&request.slug) {
        return Err(AppError::Validation(
            "slug must be a lowercase device slug".into(),
        ));
    }

    let exists = match request.category.as_str() {
        "smartphone" => {
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1
                      FROM device_models
                     WHERE slug = $1 AND publication_status = 'published'
                )
                "#,
            )
            .bind(&request.slug)
            .fetch_one(&state.db)
            .await?
        }
        "camera" => {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM camera_models WHERE slug = $1)",
            )
            .bind(&request.slug)
            .fetch_one(&state.db)
            .await?
        }
        _ => unreachable!(),
    };
    if !exists {
        return Err(AppError::NotFound);
    }

    sqlx::query(
        r#"
        INSERT INTO device_popularity_daily
            (stat_date, category, device_slug, view_count, compare_count, last_selected_at)
        VALUES (
            (now() AT TIME ZONE 'Asia/Seoul')::date,
            $1,
            $2,
            CASE WHEN $3 = 'view' THEN 1 ELSE 0 END,
            CASE WHEN $3 = 'compare' THEN 1 ELSE 0 END,
            now()
        )
        ON CONFLICT (stat_date, category, device_slug) DO UPDATE
            SET view_count = device_popularity_daily.view_count + EXCLUDED.view_count,
                compare_count = device_popularity_daily.compare_count + EXCLUDED.compare_count,
                last_selected_at = EXCLUDED.last_selected_at
        "#,
    )
    .bind(&request.category)
    .bind(&request.slug)
    .bind(&request.action)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/popular-devices",
    tag = "catalog",
    responses(
        (status = 200, body = PopularDevicesResponse),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_popular_devices(
    State(state): State<AppState>,
) -> ApiResult<Json<PopularDevicesResponse>> {
    if let Some(cached) = state.popular_devices_cache.read().await.as_ref()
        && cached.expires_at > Instant::now()
    {
        return Ok(Json(cached.response.clone()));
    }

    let (smartphones, cameras) = tokio::try_join!(
        load_popular_smartphones(&state),
        load_popular_cameras(&state)
    )?;
    let response = PopularDevicesResponse {
        items: interleave(smartphones, cameras),
        window_days: POPULARITY_WINDOW_DAYS,
        cache_ttl_seconds: CACHE_TTL.as_secs(),
    };

    *state.popular_devices_cache.write().await = Some(CachedPopularDevices {
        expires_at: Instant::now() + CACHE_TTL,
        response: response.clone(),
    });

    Ok(Json(response))
}

async fn load_popular_smartphones(state: &AppState) -> ApiResult<Vec<PopularDeviceSummary>> {
    let rows = sqlx::query_as::<_, PopularDeviceRow>(
        r#"
        WITH popularity AS (
            SELECT device_slug,
                   sum(view_count + compare_count)::bigint AS selection_count,
                   max(last_selected_at) AS last_selected_at
              FROM device_popularity_daily
             WHERE category = 'smartphone'
               AND stat_date >= (now() AT TIME ZONE 'Asia/Seoul')::date - ($1::int - 1)
             GROUP BY device_slug
        )
        SELECT dm.id,
               dm.slug,
               'smartphone'::text AS category,
               b.name AS brand,
               b.slug AS brand_slug,
               dm.name,
               dm.release_date,
               dm.market_code,
               ARRAY(
                   SELECT da.value FROM device_aliases da
                    WHERE da.device_id = dm.id AND da.kind = 'alias'
                    ORDER BY da.position
               ) AS aliases,
               ARRAY(
                   SELECT da.value FROM device_aliases da
                    WHERE da.device_id = dm.id AND da.kind = 'model_number'
                    ORDER BY da.position
               ) AS model_numbers,
               dm.image_url,
               COALESCE(p.selection_count, 0) AS selection_count
          FROM device_models dm
          JOIN brands b ON b.id = dm.brand_id
          LEFT JOIN popularity p ON p.device_slug = dm.slug
         WHERE dm.publication_status = 'published'
           AND dm.verified_at IS NOT NULL
           AND dm.release_date IS NOT NULL
           AND dm.release_date <= (now() AT TIME ZONE 'Asia/Seoul')::date
         ORDER BY (p.selection_count IS NOT NULL) DESC,
                  p.selection_count DESC NULLS LAST,
                  p.last_selected_at DESC NULLS LAST,
                  dm.release_date DESC,
                  dm.slug
         LIMIT $2
        "#,
    )
    .bind(i32::from(POPULARITY_WINDOW_DAYS))
    .bind(ITEMS_PER_CATEGORY)
    .fetch_all(&state.db)
    .await?;

    convert_rows(rows)
}

async fn load_popular_cameras(state: &AppState) -> ApiResult<Vec<PopularDeviceSummary>> {
    let rows = sqlx::query_as::<_, PopularDeviceRow>(
        r#"
        WITH popularity AS (
            SELECT device_slug,
                   sum(view_count + compare_count)::bigint AS selection_count,
                   max(last_selected_at) AS last_selected_at
              FROM device_popularity_daily
             WHERE category = 'camera'
               AND stat_date >= (now() AT TIME ZONE 'Asia/Seoul')::date - ($1::int - 1)
             GROUP BY device_slug
        )
        SELECT cm.id,
               cm.slug,
               'camera'::text AS category,
               b.name AS brand,
               b.slug AS brand_slug,
               cm.name,
               to_date(cm.release_month || '-01', 'YYYY-MM-DD') AS release_date,
               'JP'::text AS market_code,
               ARRAY[]::text[] AS aliases,
               ARRAY[]::text[] AS model_numbers,
               NULL::text AS image_url,
               COALESCE(p.selection_count, 0) AS selection_count
          FROM camera_models cm
          JOIN brands b ON b.id = cm.brand_id
          LEFT JOIN popularity p ON p.device_slug = cm.slug
         ORDER BY (p.selection_count IS NOT NULL) DESC,
                  p.selection_count DESC NULLS LAST,
                  p.last_selected_at DESC NULLS LAST,
                  cm.release_month DESC,
                  cm.slug
         LIMIT $2
        "#,
    )
    .bind(i32::from(POPULARITY_WINDOW_DAYS))
    .bind(ITEMS_PER_CATEGORY)
    .fetch_all(&state.db)
    .await?;

    convert_rows(rows)
}

#[derive(FromRow)]
struct PopularDeviceRow {
    id: Uuid,
    slug: String,
    category: String,
    brand: String,
    brand_slug: String,
    name: String,
    release_date: NaiveDate,
    market_code: String,
    aliases: Vec<String>,
    model_numbers: Vec<String>,
    image_url: Option<String>,
    selection_count: i64,
}

fn convert_rows(rows: Vec<PopularDeviceRow>) -> ApiResult<Vec<PopularDeviceSummary>> {
    rows.into_iter()
        .map(|row| {
            Ok(PopularDeviceSummary {
                id: row.id,
                slug: row.slug,
                category: row.category,
                brand: row.brand,
                brand_slug: row.brand_slug,
                name: row.name,
                release_date: row.release_date,
                market_code: row.market_code,
                aliases: row.aliases,
                model_numbers: row.model_numbers,
                image_url: row.image_url,
                selection_count: u64::try_from(row.selection_count)
                    .map_err(|_| AppError::Internal)?,
            })
        })
        .collect()
}

fn interleave(
    smartphones: Vec<PopularDeviceSummary>,
    cameras: Vec<PopularDeviceSummary>,
) -> Vec<PopularDeviceSummary> {
    let mut smartphones = smartphones.into_iter();
    let mut cameras = cameras.into_iter();
    let mut items = Vec::with_capacity((ITEMS_PER_CATEGORY * 2) as usize);

    loop {
        let smartphone = smartphones.next();
        let camera = cameras.next();
        if smartphone.is_none() && camera.is_none() {
            break;
        }
        if let Some(item) = smartphone {
            items.push(item);
        }
        if let Some(item) = camera {
            items.push(item);
        }
    }

    items
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}
