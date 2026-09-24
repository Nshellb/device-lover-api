use std::collections::{BTreeMap, HashMap, HashSet};

use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{FixedOffset, NaiveDate, Utc};
use sqlx::error::ErrorKind;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::catalog::{
    MAX_COMPARISON_DEVICES, SCHEMA_VERSION, normalize_route_identifier, normalize_search_term,
    spec_keys_for_category, specification_sections,
};
use crate::dto::{
    AliasDetail, AliasInput, CatalogBrand, CatalogSchemaResponse, ComparisonResponse,
    DeviceConfiguration, DeviceDetail, DeviceListResponse, DeviceSource, DeviceSummary,
    DeviceWriteRequest, HomeResponse, Pagination, SourceInput, SpecValue,
};
use crate::error::{ApiResult, AppError};
use crate::models::{AliasRow, ConfigurationRow, DeviceRow, SourceRow, SpecRow};
use crate::state::AppState;

const PUBLIC_DEVICE_SELECT: &str = r#"
    SELECT dm.id, dm.slug, dm.category, b.name AS brand, b.slug AS brand_slug,
           dm.name, dm.release_date, dm.market_code, dm.summary_variant_label,
           dm.image_url, dm.launch_video_url, dm.publication_status, dm.updated_at
      FROM device_models dm
      JOIN brands b ON b.id = dm.brand_id
     WHERE dm.publication_status = 'published'
       AND dm.verified_at IS NOT NULL
       AND dm.release_date IS NOT NULL
       AND dm.release_date <= $1
       AND (SELECT count(*) FROM device_spec_values sv WHERE sv.device_id = dm.id) = 27
       AND EXISTS (
           SELECT 1 FROM device_sources ds
            WHERE ds.device_id = dm.id AND ds.is_primary AND ds.checked_at IS NOT NULL
       )
       AND NOT EXISTS (
           SELECT 1
             FROM device_spec_values sv
             LEFT JOIN device_sources ds
               ON ds.id = sv.source_id AND ds.device_id = sv.device_id
            WHERE sv.device_id = dm.id
              AND sv.source_id IS NOT NULL
              AND ds.checked_at IS NULL
       )
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/catalog/schema", get(get_catalog_schema))
        .route("/api/v1/devices", get(list_devices).post(create_device))
        .route("/api/v1/devices/{identifier}", get(get_device))
        .route(
            "/api/v1/devices/by-id/{id}",
            get(get_admin_device).put(update_device),
        )
        .route(
            "/api/v1/devices/by-slug/{slug}",
            get(get_admin_device_by_slug),
        )
        .route("/api/v1/comparisons", get(get_comparison))
        .route("/api/v1/home", get(get_home))
        .route("/api/v1/admin/devices", get(list_admin_devices))
        .route("/api/v1/admin/device-brands", get(list_admin_device_brands))
}

#[utoipa::path(
    get,
    path = "/api/v1/catalog/schema",
    tag = "catalog",
    responses(
        (status = 200, body = CatalogSchemaResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_catalog_schema(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<CatalogSchemaResponse>> {
    reject_query_parameters(raw_query.as_deref())?;
    let as_of = korean_today();
    let brands = sqlx::query_as::<_, (String, String)>(&format!(
        r#"
        SELECT DISTINCT pd.brand_slug, pd.brand
          FROM ({PUBLIC_DEVICE_SELECT}) pd
         ORDER BY pd.brand_slug
        "#
    ))
    .bind(as_of)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(slug, name)| CatalogBrand { slug, name })
    .collect();

    Ok(Json(CatalogSchemaResponse {
        schema_version: SCHEMA_VERSION,
        category: "smartphone",
        max_comparison_devices: MAX_COMPARISON_DEVICES as u8,
        brands,
        sections: specification_sections(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/devices",
    tag = "catalog",
    params(
        ("q" = Option<String>, Query),
        ("category" = Option<String>, Query),
        ("brand" = Option<String>, Query),
        ("page" = Option<u32>, Query),
        ("page_size" = Option<u32>, Query),
        ("sort" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = DeviceListResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_devices(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<DeviceListResponse>> {
    let query = ListQuery::parse(raw_query.as_deref())?;
    let as_of = korean_today();
    let mut transaction = repeatable_read(&state).await?;

    let total = sqlx::query_scalar::<_, i64>(&format!(
        r#"
        SELECT count(*)
          FROM ({PUBLIC_DEVICE_SELECT}) pd
         WHERE ($2 = '' OR EXISTS (
             SELECT 1 FROM device_identifiers di
              WHERE di.device_id = pd.id AND position($2 IN di.search_key) > 0
         ))
           AND ($3::text IS NULL OR pd.brand_slug = $3)
           AND pd.category = $4
        "#
    ))
    .bind(as_of)
    .bind(&query.search_key)
    .bind(&query.brand)
    .bind(&query.category)
    .fetch_one(&mut *transaction)
    .await?;

    let rows = sqlx::query_as::<_, DeviceRow>(&format!(
        r#"
        SELECT pd.*
          FROM ({PUBLIC_DEVICE_SELECT}) pd
         WHERE ($2 = '' OR EXISTS (
             SELECT 1 FROM device_identifiers di
              WHERE di.device_id = pd.id AND position($2 IN di.search_key) > 0
         ))
           AND ($3::text IS NULL OR pd.brand_slug = $3)
           AND pd.category = $7
         ORDER BY
           CASE WHEN $2 <> '' AND $6 = 'relevance' THEN (
               SELECT min(CASE
                   WHEN di.search_key = $2 THEN 0
                   WHEN di.search_key LIKE $2 || '%' THEN 1
                   ELSE 2
               END)
                 FROM device_identifiers di
                WHERE di.device_id = pd.id AND position($2 IN di.search_key) > 0
           ) END ASC NULLS LAST,
           pd.release_date DESC, pd.name COLLATE "C", pd.slug
         LIMIT $4 OFFSET $5
        "#
    ))
    .bind(as_of)
    .bind(&query.search_key)
    .bind(&query.brand)
    .bind(i64::from(query.page_size))
    .bind(i64::from((query.page - 1) * query.page_size))
    .bind(&query.sort)
    .bind(&query.category)
    .fetch_all(&mut *transaction)
    .await?;

    let items = summaries_from_rows(&mut transaction, rows).await?;
    transaction.commit().await?;
    let total = u64::try_from(total).map_err(|_| AppError::Internal)?;
    let page_size = u64::from(query.page_size);

    Ok(Json(DeviceListResponse {
        items,
        pagination: Pagination {
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages: if total == 0 {
                0
            } else {
                total.div_ceil(page_size)
            },
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/devices/{identifier}",
    tag = "catalog",
    params(("identifier" = String, Path)),
    responses(
        (status = 200, body = DeviceDetail),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_device(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<DeviceDetail>> {
    reject_query_parameters(raw_query.as_deref())?;
    let route_key = normalize_route_identifier(&identifier)?;
    let as_of = korean_today();
    let mut transaction = repeatable_read(&state).await?;
    let id = resolve_identifier(&mut transaction, as_of, &route_key)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut details = load_details(&mut transaction, as_of, &[id]).await?;
    transaction.commit().await?;
    Ok(Json(details.pop().ok_or(AppError::Internal)?))
}

#[utoipa::path(
    get,
    path = "/api/v1/comparisons",
    tag = "catalog",
    params(("identifiers" = Vec<String>, Query, max_items = 3, min_items = 1)),
    responses(
        (status = 200, body = ComparisonResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_comparison(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<ComparisonResponse>> {
    let identifiers = parse_comparison_query(raw_query.as_deref())?;
    let route_keys = identifiers
        .iter()
        .map(|identifier| normalize_route_identifier(identifier))
        .collect::<Result<Vec<_>, _>>()?;
    let as_of = korean_today();
    let mut transaction = repeatable_read(&state).await?;

    let resolved = resolve_identifiers(&mut transaction, as_of, &route_keys).await?;
    if resolved.len() != route_keys.len() {
        return Err(AppError::NotFound);
    }

    let ids = route_keys
        .iter()
        .map(|key| resolved.get(key).copied().ok_or(AppError::NotFound))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.iter().copied().collect::<HashSet<_>>().len() != ids.len() {
        return Err(AppError::Validation(
            "identifiers must resolve to distinct devices".into(),
        ));
    }

    let devices = load_details(&mut transaction, as_of, &ids).await?;
    transaction.commit().await?;
    let canonical_path = comparison_path(&devices);

    Ok(Json(ComparisonResponse {
        devices,
        canonical_path,
        schema_version: SCHEMA_VERSION,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/home",
    tag = "catalog",
    responses(
        (status = 200, body = HomeResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_home(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<HomeResponse>> {
    reject_query_parameters(raw_query.as_deref())?;
    let as_of = korean_today();
    let mut transaction = repeatable_read(&state).await?;
    let ids = sqlx::query_scalar::<_, Uuid>(&format!(
        r#"
        SELECT pd.id FROM ({PUBLIC_DEVICE_SELECT}) pd
         ORDER BY pd.release_date DESC, pd.slug
         LIMIT 2
        "#
    ))
    .bind(as_of)
    .fetch_all(&mut *transaction)
    .await?;
    let devices = load_details(&mut transaction, as_of, &ids).await?;
    transaction.commit().await?;
    let canonical_path = (!devices.is_empty()).then(|| comparison_path(&devices));

    Ok(Json(HomeResponse {
        devices,
        canonical_path,
        schema_version: SCHEMA_VERSION,
        as_of,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/devices",
    tag = "admin",
    params(
        ("q" = Option<String>, Query, description = "Device name or model number, up to 100 characters"),
        ("search_field" = Option<String>, Query, description = "name or model_number"),
        ("brand" = Option<String>, Query, description = "Brand slug"),
        ("publication_status" = Option<String>, Query, description = "draft, published or archived"),
        ("sort" = Option<String>, Query, description = "release_date_asc or release_date_desc"),
        ("page" = Option<u32>, Query),
        ("page_size" = Option<u32>, Query)
    ),
    responses(
        (status = 200, body = DeviceListResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_admin_devices(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<DeviceListResponse>> {
    let query = AdminDeviceListQuery::parse(raw_query.as_deref())?;

    let mut transaction = state.db.begin().await?;
    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
          FROM device_models dm
          JOIN brands b ON b.id = dm.brand_id
         WHERE dm.category = 'smartphone'
           AND ($1::text IS NULL OR b.slug = $1)
           AND ($2::text IS NULL OR dm.publication_status = $2)
           AND ($3::text = '' OR
                ($4 = 'name' AND position(lower($3) IN lower(dm.name)) > 0) OR
                ($4 = 'model_number' AND EXISTS (
                    SELECT 1
                      FROM device_aliases da
                     WHERE da.device_id = dm.id
                       AND da.kind = 'model_number'
                       AND position(lower($3) IN lower(da.value)) > 0
                )))
        "#,
    )
    .bind(&query.brand)
    .bind(&query.publication_status)
    .bind(&query.search)
    .bind(&query.search_field)
    .fetch_one(&mut *transaction)
    .await?;

    let rows = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT dm.id, dm.slug, dm.category, b.name AS brand, b.slug AS brand_slug,
               dm.name, dm.release_date, dm.market_code, dm.summary_variant_label,
               dm.image_url, dm.launch_video_url, dm.publication_status, dm.updated_at
          FROM device_models dm
          JOIN brands b ON b.id = dm.brand_id
         WHERE dm.category = 'smartphone'
           AND ($1::text IS NULL OR b.slug = $1)
           AND ($2::text IS NULL OR dm.publication_status = $2)
           AND ($3::text = '' OR
                ($4 = 'name' AND position(lower($3) IN lower(dm.name)) > 0) OR
                ($4 = 'model_number' AND EXISTS (
                    SELECT 1
                      FROM device_aliases da
                     WHERE da.device_id = dm.id
                       AND da.kind = 'model_number'
                       AND position(lower($3) IN lower(da.value)) > 0
                )))
         ORDER BY
           CASE WHEN $5 = 'release_date_asc' THEN dm.release_date END ASC NULLS LAST,
           CASE WHEN $5 = 'release_date_desc' THEN dm.release_date END DESC NULLS LAST,
           dm.name COLLATE "C", dm.slug
         LIMIT $6 OFFSET $7
        "#,
    )
    .bind(&query.brand)
    .bind(&query.publication_status)
    .bind(&query.search)
    .bind(&query.search_field)
    .bind(&query.sort)
    .bind(i64::from(query.page_size))
    .bind(i64::from((query.page - 1) * query.page_size))
    .fetch_all(&mut *transaction)
    .await?;

    let items = summaries_from_rows(&mut transaction, rows).await?;
    transaction.commit().await?;
    let total = u64::try_from(total).map_err(|_| AppError::Internal)?;
    let page_size_u64 = u64::from(query.page_size);

    Ok(Json(DeviceListResponse {
        items,
        pagination: Pagination {
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages: if total == 0 {
                0
            } else {
                total.div_ceil(page_size_u64)
            },
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/device-brands",
    tag = "admin",
    responses(
        (status = 200, body = [CatalogBrand]),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn list_admin_device_brands(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<CatalogBrand>>> {
    reject_query_parameters(raw_query.as_deref())?;
    let brands = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT b.slug, b.name
          FROM brands b
          JOIN device_models dm ON dm.brand_id = b.id
         WHERE dm.category = 'smartphone'
         GROUP BY b.slug, b.name
         ORDER BY b.name COLLATE "C", b.slug
        "#,
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(slug, name)| CatalogBrand { slug, name })
    .collect();

    Ok(Json(brands))
}

#[utoipa::path(
    post,
    path = "/api/v1/devices",
    tag = "admin",
    request_body = DeviceWriteRequest,
    responses(
        (status = 201, body = DeviceDetail),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_device(
    State(state): State<AppState>,
    Json(payload): Json<DeviceWriteRequest>,
) -> ApiResult<(StatusCode, Json<DeviceDetail>)> {
    validate_write_request(&payload, "smartphone")?;
    let mut transaction = state.db.begin().await?;

    let brand_id = upsert_brand(&mut transaction, &payload.brand_slug, &payload.brand_name).await?;
    let verified_at = (payload.publication_status == "published").then(Utc::now);

    let device_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO device_models
            (brand_id, category, slug, name, market_code, release_date,
             summary_variant_label, image_url, launch_video_url, publication_status, verified_at)
        VALUES ($1, 'smartphone', $2, $3, 'KR', $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(brand_id)
    .bind(&payload.slug)
    .bind(&payload.name)
    .bind(payload.release_date)
    .bind(&payload.variant)
    .bind(&payload.image_url)
    .bind(&payload.launch_video_url)
    .bind(&payload.publication_status)
    .bind(verified_at)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_write_db_error)?;

    replace_children(&mut transaction, device_id, &payload).await?;
    let detail = load_admin_detail(&mut transaction, device_id).await?;
    transaction.commit().await?;

    Ok((StatusCode::CREATED, Json(detail)))
}

#[utoipa::path(
    get,
    path = "/api/v1/devices/by-id/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = DeviceDetail),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_admin_device(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DeviceDetail>> {
    let mut transaction = state.db.begin().await?;
    let detail = load_admin_detail(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    get,
    path = "/api/v1/devices/by-slug/{slug}",
    tag = "admin",
    params(("slug" = String, Path)),
    responses(
        (status = 200, body = DeviceDetail),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_admin_device_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<DeviceDetail>> {
    let mut transaction = state.db.begin().await?;
    // Exact match only, any publication status — this exists so the BO's
    // /{slug}/edit route works for drafts too, unlike the public identifier
    // resolver which only ever sees published devices.
    let id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM device_models WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(AppError::NotFound)?;
    let detail = load_admin_detail(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    put,
    path = "/api/v1/devices/by-id/{id}",
    tag = "admin",
    params(("id" = Uuid, Path)),
    request_body = DeviceWriteRequest,
    responses(
        (status = 200, body = DeviceDetail),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope),
        (status = 503, body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_device(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<DeviceWriteRequest>,
) -> ApiResult<Json<DeviceDetail>> {
    let mut transaction = state.db.begin().await?;

    // category is immutable after creation — validate specs against whatever
    // category the row actually has, not whatever the client happened to send.
    let existing_category =
        sqlx::query_scalar::<_, String>("SELECT category FROM device_models WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or(AppError::NotFound)?;
    validate_write_request(&payload, &existing_category)?;

    let brand_id = upsert_brand(&mut transaction, &payload.brand_slug, &payload.brand_name).await?;
    let verified_at = (payload.publication_status == "published").then(Utc::now);

    let updated = sqlx::query(
        r#"
        UPDATE device_models
           SET brand_id = $2, slug = $3, name = $4, release_date = $5,
               summary_variant_label = $6, image_url = $7, launch_video_url = $8,
               publication_status = $9, verified_at = $10, updated_at = now()
         WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(brand_id)
    .bind(&payload.slug)
    .bind(&payload.name)
    .bind(payload.release_date)
    .bind(&payload.variant)
    .bind(&payload.image_url)
    .bind(&payload.launch_video_url)
    .bind(&payload.publication_status)
    .bind(verified_at)
    .execute(&mut *transaction)
    .await
    .map_err(map_write_db_error)?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    replace_children(&mut transaction, id, &payload).await?;
    let detail = load_admin_detail(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(Json(detail))
}

fn validate_write_request(payload: &DeviceWriteRequest, category: &str) -> ApiResult<()> {
    if category != "smartphone" {
        return Err(AppError::Validation("category must be smartphone".into()));
    }
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
    if !matches!(
        payload.publication_status.as_str(),
        "draft" | "published" | "archived"
    ) {
        return Err(AppError::Validation(
            "publicationStatus must be draft, published, or archived".into(),
        ));
    }

    let expected_keys: HashSet<&str> = spec_keys_for_category(category).iter().copied().collect();
    let provided_keys: HashSet<&str> = payload.specs.keys().map(String::as_str).collect();
    if provided_keys != expected_keys {
        return Err(AppError::Validation(format!(
            "specs must include exactly the {} known keys for category {category}",
            expected_keys.len()
        )));
    }
    for (key, spec) in &payload.specs {
        if !matches!(
            spec.status.as_str(),
            "known" | "unknown" | "not_disclosed" | "not_applicable"
        ) {
            return Err(AppError::Validation(format!("spec {key}: invalid status")));
        }
        let has_raw = spec.raw.as_ref().is_some_and(|value| !value.is_null());
        if (spec.status == "known") != has_raw {
            return Err(AppError::Validation(format!(
                "spec {key}: raw must be present only when status is known"
            )));
        }
        if spec.value.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "spec {key}: value must not be empty"
            )));
        }
    }

    for alias in &payload.aliases {
        if !matches!(
            alias.kind.as_str(),
            "alias" | "model_number" | "hardware_identifier"
        ) {
            return Err(AppError::Validation(
                "alias kind must be alias, model_number, or hardware_identifier".into(),
            ));
        }
        if alias.value.trim().is_empty() {
            return Err(AppError::Validation("alias value must not be empty".into()));
        }
    }

    for config in &payload.configurations {
        if (config.ram_status == "known") != config.ram_gb.is_some() {
            return Err(AppError::Validation(
                "configuration ramGb must be present only when ramStatus is known".into(),
            ));
        }
        if config.storage_gb <= 0 {
            return Err(AppError::Validation(
                "configuration storageGb must be positive".into(),
            ));
        }
    }

    if payload
        .sources
        .iter()
        .filter(|source| source.is_primary)
        .count()
        > 1
    {
        return Err(AppError::Validation(
            "only one source may be marked primary".into(),
        ));
    }
    for source in &payload.sources {
        if source.title.trim().is_empty() {
            return Err(AppError::Validation(
                "source title must not be empty".into(),
            ));
        }
    }

    Ok(())
}

async fn upsert_brand(
    transaction: &mut Transaction<'_, Postgres>,
    slug: &str,
    name: &str,
) -> ApiResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO brands (slug, name) VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(name)
    .fetch_one(&mut **transaction)
    .await
    .map_err(map_write_db_error)
}

async fn replace_children(
    transaction: &mut Transaction<'_, Postgres>,
    device_id: Uuid,
    payload: &DeviceWriteRequest,
) -> ApiResult<()> {
    for table in [
        "device_aliases",
        "device_identifiers",
        "device_sources",
        "device_configurations",
        "device_spec_values",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE device_id = $1"))
            .bind(device_id)
            .execute(&mut **transaction)
            .await?;
    }

    for (position, alias) in payload.aliases.iter().enumerate() {
        insert_alias(transaction, device_id, alias, position as i32).await?;
    }

    let mut identifiers: BTreeMap<String, String> = BTreeMap::new();
    let raw_identifiers = std::iter::once(payload.slug.clone())
        .chain(std::iter::once(payload.name.clone()))
        .chain(payload.aliases.iter().map(|alias| alias.value.clone()));
    for raw in raw_identifiers {
        let route_key = normalize_route_identifier(&raw)?;
        let search_key = normalize_search_term(&raw);
        identifiers.entry(route_key).or_insert(search_key);
    }
    for (route_key, search_key) in identifiers {
        sqlx::query(
            "INSERT INTO device_identifiers (route_key, device_id, search_key) VALUES ($1, $2, $3)",
        )
        .bind(&route_key)
        .bind(device_id)
        .bind(&search_key)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_db_error)?;
    }

    for source in &payload.sources {
        insert_source(transaction, device_id, source).await?;
    }

    for (position, config) in payload.configurations.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO device_configurations
                (device_id, label, storage_gb, ram_gb, ram_status, position)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(device_id)
        .bind(&config.label)
        .bind(config.storage_gb)
        .bind(config.ram_gb)
        .bind(&config.ram_status)
        .bind(position as i32)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_db_error)?;
    }

    for (key, spec) in &payload.specs {
        sqlx::query(
            r#"
            INSERT INTO device_spec_values (device_id, spec_key, status, raw_value, display_value, detail)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(device_id)
        .bind(key)
        .bind(&spec.status)
        .bind(&spec.raw)
        .bind(&spec.value)
        .bind(&spec.detail)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_db_error)?;
    }

    Ok(())
}

async fn insert_alias(
    transaction: &mut Transaction<'_, Postgres>,
    device_id: Uuid,
    alias: &AliasInput,
    position: i32,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO device_aliases (device_id, value, kind, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(device_id)
    .bind(&alias.value)
    .bind(&alias.kind)
    .bind(position)
    .execute(&mut **transaction)
    .await
    .map_err(map_write_db_error)?;
    Ok(())
}

async fn insert_source(
    transaction: &mut Transaction<'_, Postgres>,
    device_id: Uuid,
    source: &SourceInput,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO device_sources (device_id, url, title, checked_at, is_primary)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(device_id)
    .bind(&source.url)
    .bind(&source.title)
    .bind(source.checked_at)
    .bind(source.is_primary)
    .execute(&mut **transaction)
    .await
    .map_err(map_write_db_error)?;
    Ok(())
}

fn map_write_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        return match db_error.kind() {
            ErrorKind::UniqueViolation => AppError::Conflict(db_error.message().to_string()),
            ErrorKind::CheckViolation
            | ErrorKind::NotNullViolation
            | ErrorKind::ForeignKeyViolation => {
                AppError::Validation(db_error.message().to_string())
            }
            _ => AppError::Database(error),
        };
    }
    AppError::Database(error)
}

/// Loads a device's full detail regardless of publication status, for the
/// admin write endpoints' response — unlike `load_details`, it doesn't
/// require the device to already meet the public-listing readiness bar.
async fn load_admin_detail(
    transaction: &mut Transaction<'_, Postgres>,
    device_id: Uuid,
) -> ApiResult<DeviceDetail> {
    let row = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT dm.id, dm.slug, dm.category, b.name AS brand, b.slug AS brand_slug,
               dm.name, dm.release_date, dm.market_code, dm.summary_variant_label,
               dm.image_url, dm.launch_video_url, dm.publication_status, dm.updated_at
          FROM device_models dm
          JOIN brands b ON b.id = dm.brand_id
         WHERE dm.id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(AppError::NotFound)?;

    let ids = [device_id];
    let alias_list = load_aliases(transaction, &ids)
        .await?
        .remove(&device_id)
        .unwrap_or_default();
    let device_sources = group_sources(transaction, &ids)
        .await?
        .remove(&device_id)
        .unwrap_or_default();
    let source_ids: HashSet<Uuid> = device_sources.iter().map(|source| source.id).collect();
    let source_url = device_sources
        .iter()
        .find(|source| source.is_primary)
        .or_else(|| device_sources.first())
        .map(|source| source.url.clone())
        .unwrap_or_default();

    let mut response_specs = BTreeMap::new();
    for spec in group_specs(transaction, &ids)
        .await?
        .remove(&device_id)
        .unwrap_or_default()
    {
        let unsupported = spec
            .raw_value
            .as_ref()
            .and_then(|raw| raw.get("supported"))
            .and_then(|supported| supported.as_bool())
            == Some(false);
        response_specs.insert(
            spec.spec_key,
            SpecValue {
                muted: spec.status != "known" || unsupported,
                status: spec.status,
                raw: spec.raw_value,
                value: spec.display_value,
                detail: spec.detail,
                source_id: spec.source_id.filter(|id| source_ids.contains(id)),
            },
        );
    }

    let configurations = group_configurations(transaction, &ids)
        .await?
        .remove(&device_id)
        .unwrap_or_default();

    let alias_details = alias_list
        .iter()
        .map(|alias| AliasDetail {
            value: alias.value.clone(),
            kind: alias.kind.clone(),
        })
        .collect();
    let summary = summary_from_row(&row, Some(&alias_list));

    Ok(DeviceDetail {
        summary,
        variant: row.summary_variant_label,
        configurations,
        source_url,
        sources: device_sources,
        specs: response_specs,
        updated_at: row.updated_at,
        launch_video_url: row.launch_video_url,
        alias_details,
    })
}

async fn repeatable_read(state: &AppState) -> ApiResult<Transaction<'_, Postgres>> {
    let mut transaction = state.db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await?;
    Ok(transaction)
}

async fn resolve_identifier(
    transaction: &mut Transaction<'_, Postgres>,
    as_of: NaiveDate,
    route_key: &str,
) -> ApiResult<Option<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(&format!(
        r#"
        SELECT pd.id
          FROM ({PUBLIC_DEVICE_SELECT}) pd
          JOIN device_identifiers di ON di.device_id = pd.id
         WHERE di.route_key = $2
        "#
    ))
    .bind(as_of)
    .bind(route_key)
    .fetch_optional(&mut **transaction)
    .await?)
}

async fn resolve_identifiers(
    transaction: &mut Transaction<'_, Postgres>,
    as_of: NaiveDate,
    route_keys: &[String],
) -> ApiResult<HashMap<String, Uuid>> {
    let rows = sqlx::query_as::<_, (String, Uuid)>(&format!(
        r#"
        SELECT di.route_key, pd.id
          FROM ({PUBLIC_DEVICE_SELECT}) pd
          JOIN device_identifiers di ON di.device_id = pd.id
         WHERE di.route_key = ANY($2)
        "#
    ))
    .bind(as_of)
    .bind(route_keys)
    .fetch_all(&mut **transaction)
    .await?;
    Ok(rows.into_iter().collect())
}

async fn summaries_from_rows(
    transaction: &mut Transaction<'_, Postgres>,
    rows: Vec<DeviceRow>,
) -> ApiResult<Vec<DeviceSummary>> {
    let ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let aliases = load_aliases(transaction, &ids).await?;
    Ok(rows
        .into_iter()
        .map(|row| summary_from_row(&row, aliases.get(&row.id)))
        .collect())
}

async fn load_details(
    transaction: &mut Transaction<'_, Postgres>,
    as_of: NaiveDate,
    ids: &[Uuid],
) -> ApiResult<Vec<DeviceDetail>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, DeviceRow>(&format!(
        r#"
        SELECT pd.* FROM ({PUBLIC_DEVICE_SELECT}) pd WHERE pd.id = ANY($2)
        "#
    ))
    .bind(as_of)
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await?;
    let mut rows_by_id = rows
        .into_iter()
        .map(|row| (row.id, row))
        .collect::<HashMap<_, _>>();
    if rows_by_id.len() != ids.len() {
        return Err(AppError::NotFound);
    }

    let aliases = load_aliases(transaction, ids).await?;
    let mut configurations = group_configurations(transaction, ids).await?;
    let mut sources = group_sources(transaction, ids).await?;
    let mut specs = group_specs(transaction, ids).await?;
    let mut details = Vec::with_capacity(ids.len());

    for id in ids {
        let row = rows_by_id.remove(id).ok_or(AppError::Internal)?;
        let expected_keys = spec_keys_for_category(&row.category)
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let device_sources = sources.remove(id).unwrap_or_default();
        let primary_sources = device_sources
            .iter()
            .filter(|source| source.is_primary && source.checked_at.is_some())
            .collect::<Vec<_>>();
        if primary_sources.len() != 1 {
            return Err(AppError::Internal);
        }
        let source_url = primary_sources[0].url.clone();
        let source_ids = device_sources
            .iter()
            .map(|source| source.id)
            .collect::<HashSet<_>>();
        let spec_rows = specs.remove(id).unwrap_or_default();
        if spec_rows.len() != expected_keys.len()
            || spec_rows
                .iter()
                .map(|spec| spec.spec_key.as_str())
                .collect::<HashSet<_>>()
                != expected_keys
        {
            return Err(AppError::Internal);
        }

        let mut response_specs = BTreeMap::new();
        for spec in spec_rows {
            if spec
                .source_id
                .is_some_and(|source_id| !source_ids.contains(&source_id))
                || (spec.status == "known") != spec.raw_value.is_some()
            {
                return Err(AppError::Internal);
            }
            let unsupported = spec
                .raw_value
                .as_ref()
                .and_then(|raw| raw.get("supported"))
                .and_then(|supported| supported.as_bool())
                == Some(false);
            response_specs.insert(
                spec.spec_key,
                SpecValue {
                    muted: spec.status != "known" || unsupported,
                    status: spec.status,
                    raw: spec.raw_value,
                    value: spec.display_value,
                    detail: spec.detail,
                    source_id: spec.source_id,
                },
            );
        }

        let alias_details = aliases
            .get(id)
            .map(|rows| {
                rows.iter()
                    .map(|alias| AliasDetail {
                        value: alias.value.clone(),
                        kind: alias.kind.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let summary = summary_from_row(&row, aliases.get(id));
        details.push(DeviceDetail {
            summary,
            variant: row.summary_variant_label,
            configurations: configurations.remove(id).unwrap_or_default(),
            source_url,
            sources: device_sources,
            specs: response_specs,
            updated_at: row.updated_at,
            launch_video_url: row.launch_video_url,
            alias_details,
        });
    }

    Ok(details)
}

async fn load_aliases(
    transaction: &mut Transaction<'_, Postgres>,
    ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Vec<AliasRow>>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query_as::<_, AliasRow>(
        "SELECT device_id, value, kind FROM device_aliases WHERE device_id = ANY($1) ORDER BY device_id, position",
    )
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await?;
    Ok(group_by_device(rows, |row| row.device_id))
}

async fn group_configurations(
    transaction: &mut Transaction<'_, Postgres>,
    ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Vec<DeviceConfiguration>>> {
    let rows = sqlx::query_as::<_, ConfigurationRow>(
        "SELECT id, device_id, label, storage_gb, ram_gb, ram_status FROM device_configurations WHERE device_id = ANY($1) ORDER BY device_id, position",
    )
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await?;
    let mut result = HashMap::<Uuid, Vec<DeviceConfiguration>>::new();
    for row in rows {
        result
            .entry(row.device_id)
            .or_default()
            .push(DeviceConfiguration {
                id: row.id,
                label: row.label,
                storage_gb: row.storage_gb,
                ram_gb: row.ram_gb,
                ram_status: row.ram_status,
            });
    }
    Ok(result)
}

async fn group_sources(
    transaction: &mut Transaction<'_, Postgres>,
    ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Vec<DeviceSource>>> {
    let rows = sqlx::query_as::<_, SourceRow>(
        "SELECT id, device_id, url, title, checked_at, is_primary FROM device_sources WHERE device_id = ANY($1) ORDER BY device_id, is_primary DESC, created_at, id",
    )
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await?;
    let mut result = HashMap::<Uuid, Vec<DeviceSource>>::new();
    for row in rows {
        result.entry(row.device_id).or_default().push(DeviceSource {
            id: row.id,
            url: row.url,
            title: row.title,
            checked_at: row.checked_at,
            is_primary: row.is_primary,
        });
    }
    Ok(result)
}

async fn group_specs(
    transaction: &mut Transaction<'_, Postgres>,
    ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Vec<SpecRow>>> {
    let rows = sqlx::query_as::<_, SpecRow>(
        "SELECT device_id, spec_key, status, raw_value, display_value, detail, source_id FROM device_spec_values WHERE device_id = ANY($1)",
    )
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await?;
    Ok(group_by_device(rows, |row| row.device_id))
}

fn group_by_device<T>(rows: Vec<T>, id: impl Fn(&T) -> Uuid) -> HashMap<Uuid, Vec<T>> {
    let mut result = HashMap::<Uuid, Vec<T>>::new();
    for row in rows {
        result.entry(id(&row)).or_default().push(row);
    }
    result
}

fn summary_from_row(row: &DeviceRow, aliases: Option<&Vec<AliasRow>>) -> DeviceSummary {
    let aliases = aliases.map(Vec::as_slice).unwrap_or_default();
    DeviceSummary {
        id: row.id,
        slug: row.slug.clone(),
        category: row.category.clone(),
        brand: row.brand.clone(),
        brand_slug: row.brand_slug.clone(),
        name: row.name.clone(),
        release_date: row.release_date,
        market_code: row.market_code.clone(),
        aliases: aliases.iter().map(|alias| alias.value.clone()).collect(),
        model_numbers: aliases
            .iter()
            .filter(|alias| matches!(alias.kind.as_str(), "model_number" | "hardware_identifier"))
            .map(|alias| alias.value.clone())
            .collect(),
        image_url: row.image_url.clone(),
        publication_status: row.publication_status.clone(),
    }
}

fn comparison_path(devices: &[DeviceDetail]) -> String {
    format!(
        "/{}",
        devices
            .iter()
            .map(|device| device.summary.slug.as_str())
            .collect::<Vec<_>>()
            .join("-vs-")
    )
}

fn korean_today() -> NaiveDate {
    Utc::now()
        .with_timezone(&FixedOffset::east_opt(9 * 60 * 60).expect("valid KST offset"))
        .date_naive()
}

fn reject_query_parameters(raw_query: Option<&str>) -> ApiResult<()> {
    if parse_query_pairs(raw_query)?.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(
            "this endpoint does not accept query parameters".into(),
        ))
    }
}

fn parse_comparison_query(raw_query: Option<&str>) -> ApiResult<Vec<String>> {
    let pairs = parse_query_pairs(raw_query)?;
    if pairs.iter().any(|(key, _)| key != "identifiers") {
        return Err(AppError::Validation("unknown query parameter".into()));
    }
    let identifiers = pairs
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    if !(1..=MAX_COMPARISON_DEVICES).contains(&identifiers.len()) {
        return Err(AppError::Validation(
            "identifiers must contain 1 to 3 values".into(),
        ));
    }
    Ok(identifiers)
}

fn parse_query_pairs(raw_query: Option<&str>) -> ApiResult<Vec<(String, String)>> {
    let Some(raw_query) = raw_query.filter(|query| !query.is_empty()) else {
        return Ok(Vec::new());
    };
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
    Ok(url::form_urlencoded::parse(bytes)
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect())
}

struct AdminDeviceListQuery {
    search: String,
    search_field: String,
    brand: Option<String>,
    publication_status: Option<String>,
    sort: String,
    page: u32,
    page_size: u32,
}

impl AdminDeviceListQuery {
    fn parse(raw_query: Option<&str>) -> ApiResult<Self> {
        let mut values = HashMap::<String, String>::new();
        for (key, value) in parse_query_pairs(raw_query)? {
            if !matches!(
                key.as_str(),
                "q" | "search_field"
                    | "brand"
                    | "publication_status"
                    | "sort"
                    | "page"
                    | "page_size"
                    | "category"
            ) {
                return Err(AppError::Validation(format!(
                    "unknown query parameter: {key}"
                )));
            }
            if values.insert(key.clone(), value).is_some() {
                return Err(AppError::Validation(format!(
                    "query parameter must not be repeated: {key}"
                )));
            }
        }

        let category = values
            .remove("category")
            .unwrap_or_else(|| "smartphone".into());
        if category != "smartphone" {
            return Err(AppError::Validation("category must be smartphone".into()));
        }

        let search = values.remove("q").unwrap_or_default().trim().to_owned();
        if search.chars().count() > 100 || search.chars().any(char::is_control) {
            return Err(AppError::Validation(
                "q must be at most 100 non-control characters".into(),
            ));
        }
        let search_field = values
            .remove("search_field")
            .unwrap_or_else(|| "name".into());
        if !matches!(search_field.as_str(), "name" | "model_number") {
            return Err(AppError::Validation(
                "search_field must be name or model_number".into(),
            ));
        }

        let brand = values.remove("brand").filter(|value| !value.is_empty());
        if brand.as_deref().is_some_and(|value| !valid_slug(value, 80)) {
            return Err(AppError::Validation(
                "brand must be a lowercase slug".into(),
            ));
        }

        let publication_status = values
            .remove("publication_status")
            .filter(|value| !value.is_empty());
        if publication_status
            .as_deref()
            .is_some_and(|value| !matches!(value, "draft" | "published" | "archived"))
        {
            return Err(AppError::Validation(
                "publication_status must be draft, published or archived".into(),
            ));
        }

        let sort = values
            .remove("sort")
            .unwrap_or_else(|| "release_date_desc".into());
        if !matches!(sort.as_str(), "release_date_asc" | "release_date_desc") {
            return Err(AppError::Validation(
                "sort must be release_date_asc or release_date_desc".into(),
            ));
        }

        Ok(Self {
            search,
            search_field,
            brand,
            publication_status,
            sort,
            page: parse_bounded_u32(values.remove("page"), "page", 1, 10_000, 1)?,
            page_size: parse_bounded_u32(values.remove("page_size"), "page_size", 1, 100, 20)?,
        })
    }
}

struct ListQuery {
    search_key: String,
    brand: Option<String>,
    category: String,
    page: u32,
    page_size: u32,
    sort: String,
}

impl ListQuery {
    fn parse(raw_query: Option<&str>) -> ApiResult<Self> {
        let mut values = HashMap::<String, String>::new();
        for (key, value) in parse_query_pairs(raw_query)? {
            if !matches!(
                key.as_str(),
                "q" | "category" | "brand" | "page" | "page_size" | "sort"
            ) {
                return Err(AppError::Validation(format!(
                    "unknown query parameter: {key}"
                )));
            }
            if values.insert(key.clone(), value).is_some() {
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
        let search_key = normalize_search_term(&q);
        let category = values
            .remove("category")
            .unwrap_or_else(|| "smartphone".into());
        if category != "smartphone" {
            return Err(AppError::Validation("category must be smartphone".into()));
        }
        let brand = values.remove("brand");
        if brand.as_deref().is_some_and(|brand| !valid_slug(brand, 80)) {
            return Err(AppError::Validation(
                "brand must be a lowercase slug".into(),
            ));
        }
        let page = parse_bounded_u32(values.remove("page"), "page", 1, 10_000, 1)?;
        let page_size = parse_bounded_u32(values.remove("page_size"), "page_size", 1, 100, 20)?;
        let sort = values.remove("sort").unwrap_or_else(|| {
            if search_key.is_empty() {
                "release_date_desc".into()
            } else {
                "relevance".into()
            }
        });
        if !matches!(sort.as_str(), "relevance" | "release_date_desc") {
            return Err(AppError::Validation(
                "sort must be relevance or release_date_desc".into(),
            ));
        }

        Ok(Self {
            search_key,
            brand,
            category,
            page,
            page_size,
            sort,
        })
    }
}

fn parse_bounded_u32(
    value: Option<String>,
    name: &str,
    minimum: u32,
    maximum: u32,
    default: u32,
) -> ApiResult<u32> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = value
        .parse::<u32>()
        .map_err(|_| AppError::Validation(format!("{name} must be an integer")))?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(AppError::Validation(format!(
            "{name} must be between {minimum} and {maximum}"
        )));
    }
    Ok(parsed)
}

fn valid_slug(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_query_rejects_unknown_and_duplicate_parameters() {
        assert!(ListQuery::parse(Some("unknown=1")).is_err());
        assert!(ListQuery::parse(Some("page=1&page=2")).is_err());
    }

    #[test]
    fn comparison_query_preserves_comma_and_encoded_plus() {
        let values =
            parse_comparison_query(Some("identifiers=iphone19%2C3&identifiers=galaxy-s24%2B"))
                .unwrap();
        assert_eq!(values, ["iphone19,3", "galaxy-s24+"]);
    }

    #[test]
    fn literal_plus_decodes_as_space_by_url_form_rules() {
        let values = parse_comparison_query(Some("identifiers=galaxy-s24+")).unwrap();
        assert_eq!(values, ["galaxy-s24 "]);
    }

    #[test]
    fn malformed_percent_encoding_is_rejected() {
        assert!(parse_comparison_query(Some("identifiers=phone%2")).is_err());
    }
}
