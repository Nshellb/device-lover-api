use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type ApiResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("authentication required or invalid credentials")]
    Unauthorized,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal error")]
    Internal,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct ErrorEnvelope {
    error: ErrorBody,
}

impl ErrorEnvelope {
    fn new(code: &'static str, message: String) -> Self {
        Self {
            error: ErrorBody { code, message },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_string(),
            ),
            AppError::Validation(message) => {
                (StatusCode::BAD_REQUEST, "validation_error", message.clone())
            }
            AppError::Conflict(message) => (StatusCode::CONFLICT, "conflict", message.clone()),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required or invalid credentials".to_string(),
            ),
            AppError::Database(error) => {
                tracing::error!(error = ?error, "database request failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "database_unavailable",
                    "database unavailable".to_string(),
                )
            }
            AppError::Internal => {
                tracing::error!(error = ?self, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(ErrorEnvelope::new(code, message))).into_response()
    }
}

pub async fn fallback() -> AppError {
    AppError::NotFound
}
