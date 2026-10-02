use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::{AdminWirelessTechnology, WirelessTechnologyInput};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::WirelessTechnologyRow;
use crate::routes::catalog::{map_write_db_error, reject_query_parameters};
use crate::state::AppState;

const SELECT_WIRELESS_TECHNOLOGY: &str =
    "SELECT id, category, value, label, sort_order FROM wireless_technologies";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/wireless-technologies",
            get(list_wireless_technologies).post(create_wireless_technology),
        )
        .route(
            "/api/v1/wireless-technologies/{id}",
            get(get_wireless_technology)
                .put(update_wireless_technology)
                .delete(delete_wireless_technology),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/wireless-technologies",
    tag = "admin",
    responses(
        (status = 200, body = [AdminWirelessTechnology]),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_wireless_technologies(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<AdminWirelessTechnology>>> {
    reject_query_parameters(raw_query.as_deref())?;
    let rows = sqlx::query_as::<_, WirelessTechnologyRow>(&format!(
        r#"{SELECT_WIRELESS_TECHNOLOGY}
           ORDER BY CASE category
               WHEN 'network' THEN 0
               WHEN 'wifi' THEN 1
               WHEN 'bluetooth' THEN 2
               WHEN 'uwb' THEN 3
               WHEN 'nfc' THEN 4
           END, sort_order, label COLLATE "C""#,
    ))
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(AdminWirelessTechnology::from)
    .collect();
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/wireless-technologies",
    tag = "admin",
    request_body = WirelessTechnologyInput,
    responses(
        (status = 201, body = AdminWirelessTechnology),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_wireless_technology(
    State(state): State<AppState>,
    AppJson(payload): AppJson<WirelessTechnologyInput>,
) -> ApiResult<(StatusCode, Json<AdminWirelessTechnology>)> {
    validate_wireless_technology_input(&payload)?;
    let row = sqlx::query_as::<_, WirelessTechnologyRow>(
        r#"INSERT INTO wireless_technologies (category, value, label, sort_order)
           VALUES ($1, $2, $3, $4)
           RETURNING id, category, value, label, sort_order"#,
    )
    .bind(&payload.category)
    .bind(payload.value.trim())
    .bind(payload.label.trim())
    .bind(payload.sort_order)
    .fetch_one(&state.db)
    .await
    .map_err(map_write_db_error)?;
    Ok((
        StatusCode::CREATED,
        Json(AdminWirelessTechnology::from(row)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/wireless-technologies/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = AdminWirelessTechnology),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_wireless_technology(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AdminWirelessTechnology>> {
    let row = sqlx::query_as::<_, WirelessTechnologyRow>(&format!(
        "{SELECT_WIRELESS_TECHNOLOGY} WHERE id = $1",
    ))
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(AdminWirelessTechnology::from(row)))
}

#[utoipa::path(
    put,
    path = "/api/v1/wireless-technologies/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    request_body = WirelessTechnologyInput,
    responses(
        (status = 200, body = AdminWirelessTechnology),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_wireless_technology(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AppJson(payload): AppJson<WirelessTechnologyInput>,
) -> ApiResult<Json<AdminWirelessTechnology>> {
    validate_wireless_technology_input(&payload)?;
    let row = sqlx::query_as::<_, WirelessTechnologyRow>(
        r#"UPDATE wireless_technologies
           SET category = $2, value = $3, label = $4, sort_order = $5, updated_at = now()
           WHERE id = $1
           RETURNING id, category, value, label, sort_order"#,
    )
    .bind(id)
    .bind(&payload.category)
    .bind(payload.value.trim())
    .bind(payload.label.trim())
    .bind(payload.sort_order)
    .fetch_optional(&state.db)
    .await
    .map_err(map_write_db_error)?
    .ok_or(AppError::NotFound)?;
    Ok(Json(AdminWirelessTechnology::from(row)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/wireless-technologies/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204, description = "Wireless technology deleted"),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn delete_wireless_technology(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM wireless_technologies WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate_wireless_technology_input(payload: &WirelessTechnologyInput) -> ApiResult<()> {
    if !matches!(
        payload.category.as_str(),
        "network" | "wifi" | "bluetooth" | "uwb" | "nfc"
    ) {
        return Err(AppError::Validation(
            "invalid wireless technology category".into(),
        ));
    }
    if payload.value.trim().is_empty() || payload.value.trim().chars().count() > 80 {
        return Err(AppError::Validation(
            "value must be between 1 and 80 characters".into(),
        ));
    }
    if payload.label.trim().is_empty() || payload.label.trim().chars().count() > 160 {
        return Err(AppError::Validation(
            "label must be between 1 and 160 characters".into(),
        ));
    }
    if !(0..=100_000).contains(&payload.sort_order) {
        return Err(AppError::Validation(
            "sortOrder must be between 0 and 100000".into(),
        ));
    }
    Ok(())
}
