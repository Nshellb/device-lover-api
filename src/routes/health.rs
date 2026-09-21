use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service and database are healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn health(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let _: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&state.db).await?;

    Ok(Json(HealthResponse {
        status: "ok",
        database: "ok",
    }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
