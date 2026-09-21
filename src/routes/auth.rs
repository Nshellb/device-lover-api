use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::auth::{encode_token, verify_password};
use crate::dto::{LoginRequest, TokenResponse};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::User;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login succeeded", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    AppJson(request): AppJson<LoginRequest>,
) -> ApiResult<Json<TokenResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&request.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&request.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    let token = encode_token(user.id, &state.settings.jwt_secret)?;

    Ok(Json(TokenResponse { token }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/login", post(login))
}
