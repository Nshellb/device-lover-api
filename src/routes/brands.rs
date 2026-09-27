use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::{AdminBrand, BrandInput};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::BrandRow;
use crate::routes::catalog::{map_write_db_error, reject_query_parameters, valid_slug};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/brands", get(list_brands).post(create_brand))
        .route(
            "/api/v1/brands/{id}",
            get(get_brand).put(update_brand).delete(delete_brand),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/brands",
    tag = "admin",
    responses(
        (status = 200, body = [AdminBrand]),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_brands(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<AdminBrand>>> {
    reject_query_parameters(raw_query.as_deref())?;
    let brands = sqlx::query_as::<_, BrandRow>(
        r#"SELECT id, slug, name FROM brands ORDER BY name COLLATE "C", slug"#,
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(AdminBrand::from)
    .collect();
    Ok(Json(brands))
}

#[utoipa::path(
    post,
    path = "/api/v1/brands",
    tag = "admin",
    request_body = BrandInput,
    responses(
        (status = 201, body = AdminBrand),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_brand(
    State(state): State<AppState>,
    AppJson(payload): AppJson<BrandInput>,
) -> ApiResult<(StatusCode, Json<AdminBrand>)> {
    validate_brand_input(&payload)?;
    let row = sqlx::query_as::<_, BrandRow>(
        "INSERT INTO brands (slug, name) VALUES ($1, $2) RETURNING id, slug, name",
    )
    .bind(&payload.slug)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(map_write_db_error)?;
    Ok((StatusCode::CREATED, Json(AdminBrand::from(row))))
}

#[utoipa::path(
    get,
    path = "/api/v1/brands/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = AdminBrand),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_brand(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AdminBrand>> {
    let row = sqlx::query_as::<_, BrandRow>("SELECT id, slug, name FROM brands WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(AdminBrand::from(row)))
}

#[utoipa::path(
    put,
    path = "/api/v1/brands/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    request_body = BrandInput,
    responses(
        (status = 200, body = AdminBrand),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_brand(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AppJson(payload): AppJson<BrandInput>,
) -> ApiResult<Json<AdminBrand>> {
    validate_brand_input(&payload)?;
    let row = sqlx::query_as::<_, BrandRow>(
        "UPDATE brands SET slug = $2, name = $3 WHERE id = $1 RETURNING id, slug, name",
    )
    .bind(id)
    .bind(&payload.slug)
    .bind(&payload.name)
    .fetch_optional(&state.db)
    .await
    .map_err(map_write_db_error)?
    .ok_or(AppError::NotFound)?;
    Ok(Json(AdminBrand::from(row)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/brands/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204, description = "Brand deleted"),
        (status = 400, description = "Brand is still referenced by a device or camera", body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn delete_brand(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM brands WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(map_write_db_error)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate_brand_input(payload: &BrandInput) -> ApiResult<()> {
    if !valid_slug(&payload.slug, 80) {
        return Err(AppError::Validation("slug must be a lowercase slug".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    Ok(())
}
