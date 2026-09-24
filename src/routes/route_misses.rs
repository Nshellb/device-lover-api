use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::dto::{RouteMissCreateRequest, RouteMissSummary, RouteMissSummaryResponse};
use crate::error::{ApiResult, AppError};
use crate::state::AppState;

const DEFAULT_WINDOW_DAYS: u16 = 30;
const DEFAULT_LIMIT: u16 = 50;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/route-misses", post(record_route_miss))
        .route("/api/v1/admin/route-misses", get(list_route_miss_summaries))
}

#[utoipa::path(
    post,
    path = "/api/v1/route-misses",
    tag = "analytics",
    request_body = RouteMissCreateRequest,
    responses(
        (status = 204, description = "Route miss recorded"),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn record_route_miss(
    State(state): State<AppState>,
    Json(request): Json<RouteMissCreateRequest>,
) -> ApiResult<StatusCode> {
    let requested_path = validate_required(&request.requested_path, "requestedPath", 512)?;
    if !requested_path.starts_with('/') {
        return Err(AppError::Validation(
            "requestedPath must start with /".into(),
        ));
    }

    let referrer = validate_optional(request.referrer, "referrer", 1024)?;
    let locale = validate_optional(request.locale, "locale", 35)?;
    let viewport_class = validate_optional(request.viewport_class, "viewportClass", 7)?;
    if let Some(value) = viewport_class.as_deref()
        && !matches!(value, "mobile" | "tablet" | "desktop")
    {
        return Err(AppError::Validation(
            "viewportClass must be mobile, tablet or desktop".into(),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO route_miss_events
            (requested_path, referrer, locale, viewport_class)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(requested_path)
    .bind(referrer)
    .bind(locale)
    .bind(viewport_class)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/route-misses",
    tag = "admin",
    params(
        ("days" = Option<u16>, Query, description = "1–365; defaults to 30"),
        ("limit" = Option<u16>, Query, description = "1–100; defaults to 50")
    ),
    responses(
        (status = 200, body = RouteMissSummaryResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_route_miss_summaries(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<RouteMissSummaryResponse>> {
    let (days, limit) = parse_summary_query(raw_query.as_deref())?;
    let rows = sqlx::query_as::<_, RouteMissSummaryRow>(
        r#"
        SELECT requested_path,
               count(*) AS hits,
               min(occurred_at) AS first_seen_at,
               max(occurred_at) AS last_seen_at
          FROM route_miss_events
         WHERE occurred_at >= now() - ($1::double precision * interval '1 day')
         GROUP BY requested_path
         ORDER BY hits DESC, last_seen_at DESC, requested_path
         LIMIT $2
        "#,
    )
    .bind(i32::from(days))
    .bind(i64::from(limit))
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            Ok(RouteMissSummary {
                requested_path: row.requested_path,
                hits: u64::try_from(row.hits).map_err(|_| AppError::Internal)?,
                first_seen_at: row.first_seen_at,
                last_seen_at: row.last_seen_at,
            })
        })
        .collect::<ApiResult<Vec<_>>>()?;

    Ok(Json(RouteMissSummaryResponse {
        items,
        window_days: days,
    }))
}

#[derive(FromRow)]
struct RouteMissSummaryRow {
    requested_path: String,
    hits: i64,
    first_seen_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
}

fn validate_required(value: &str, field: &str, max_chars: usize) -> ApiResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation(format!(
            "{field} must contain 1–{max_chars} non-control characters"
        )));
    }
    Ok(value.to_owned())
}

fn validate_optional(
    value: Option<String>,
    field: &str,
    max_chars: usize,
) -> ApiResult<Option<String>> {
    value
        .map(|value| validate_required(&value, field, max_chars))
        .transpose()
}

fn parse_summary_query(raw_query: Option<&str>) -> ApiResult<(u16, u16)> {
    let mut days = None;
    let mut limit = None;

    for (key, value) in url::form_urlencoded::parse(raw_query.unwrap_or_default().as_bytes()) {
        let target = match key.as_ref() {
            "days" => &mut days,
            "limit" => &mut limit,
            _ => {
                return Err(AppError::Validation(format!(
                    "unsupported query parameter: {key}"
                )));
            }
        };
        if target.is_some() {
            return Err(AppError::Validation(format!(
                "query parameter may only appear once: {key}"
            )));
        }
        *target = Some(
            value
                .parse::<u16>()
                .map_err(|_| AppError::Validation(format!("{key} must be an integer")))?,
        );
    }

    let days = days.unwrap_or(DEFAULT_WINDOW_DAYS);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=365).contains(&days) {
        return Err(AppError::Validation(
            "days must be between 1 and 365".into(),
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(AppError::Validation(
            "limit must be between 1 and 100".into(),
        ));
    }
    Ok((days, limit))
}
