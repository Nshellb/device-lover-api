use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::{AdminSoftwareVersion, SoftwareVersionInput};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::SoftwareVersionRow;
use crate::routes::catalog::{map_write_db_error, reject_query_parameters};
use crate::state::AppState;

const SELECT_SOFTWARE_VERSION: &str =
    "SELECT id, category, value, label, sort_order FROM software_versions";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/software-versions",
            get(list_software_versions).post(create_software_version),
        )
        .route(
            "/api/v1/software-versions/{id}",
            get(get_software_version)
                .put(update_software_version)
                .delete(delete_software_version),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/software-versions",
    tag = "admin",
    responses(
        (status = 200, body = [AdminSoftwareVersion]),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_software_versions(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<AdminSoftwareVersion>>> {
    reject_query_parameters(raw_query.as_deref())?;
    let rows = sqlx::query_as::<_, SoftwareVersionRow>(&format!(
        r#"{SELECT_SOFTWARE_VERSION}
           ORDER BY CASE category WHEN 'os' THEN 0 WHEN 'ux' THEN 1 END,
               sort_order, label COLLATE "C""#,
    ))
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(AdminSoftwareVersion::from)
    .collect();
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/software-versions",
    tag = "admin",
    request_body = SoftwareVersionInput,
    responses(
        (status = 201, body = AdminSoftwareVersion),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_software_version(
    State(state): State<AppState>,
    AppJson(payload): AppJson<SoftwareVersionInput>,
) -> ApiResult<(StatusCode, Json<AdminSoftwareVersion>)> {
    validate_software_version_input(&payload)?;
    let row = sqlx::query_as::<_, SoftwareVersionRow>(
        r#"INSERT INTO software_versions (category, value, label, sort_order)
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
        Json(AdminSoftwareVersion::from(row)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/software-versions/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = AdminSoftwareVersion),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_software_version(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AdminSoftwareVersion>> {
    let row = sqlx::query_as::<_, SoftwareVersionRow>(&format!(
        "{SELECT_SOFTWARE_VERSION} WHERE id = $1",
    ))
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(AdminSoftwareVersion::from(row)))
}

#[utoipa::path(
    put,
    path = "/api/v1/software-versions/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    request_body = SoftwareVersionInput,
    responses(
        (status = 200, body = AdminSoftwareVersion),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_software_version(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AppJson(payload): AppJson<SoftwareVersionInput>,
) -> ApiResult<Json<AdminSoftwareVersion>> {
    validate_software_version_input(&payload)?;
    let row = sqlx::query_as::<_, SoftwareVersionRow>(
        r#"UPDATE software_versions
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
    Ok(Json(AdminSoftwareVersion::from(row)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/software-versions/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204, description = "Software version deleted"),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn delete_software_version(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM software_versions WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate_software_version_input(payload: &SoftwareVersionInput) -> ApiResult<()> {
    if !matches!(
        payload.category.as_str(),
        "os" | "ux"
    ) {
        return Err(AppError::Validation(
            "invalid software version category".into(),
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
