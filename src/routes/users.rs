use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::error::{ApiResult, AppError};
use crate::extract::AppJson;
use crate::models::User;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid user data", body = crate::error::ErrorEnvelope),
        (status = 409, description = "Email already exists", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn create_user(
    State(state): State<AppState>,
    AppJson(request): AppJson<CreateUserRequest>,
) -> ApiResult<(StatusCode, Json<UserResponse>)> {
    request.validate()?;
    let password_hash = crate::auth::hash_password(&request.password)?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&request.email)
    .bind(&request.name)
    .bind(password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(map_user_write_error)?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    responses(
        (status = 200, description = "Users listed", body = [UserResponse])
    )
)]
pub(crate) async fn list_users(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at")
        .fetch_all(&state.db)
        .await?;
    let response = users.into_iter().map(UserResponse::from).collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    patch,
    path = "/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 400, description = "Invalid user data", body = crate::error::ErrorEnvelope),
        (status = 401, description = "Authentication required or invalid", body = crate::error::ErrorEnvelope),
        (status = 404, description = "User not found", body = crate::error::ErrorEnvelope),
        (status = 409, description = "Email already exists", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn update_user(
    State(state): State<AppState>,
    crate::auth::AuthUser(_user_id): crate::auth::AuthUser,
    Path(id): Path<Uuid>,
    AppJson(request): AppJson<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    request.validate()?;

    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET email = COALESCE($1, email), name = COALESCE($2, name), \
         updated_at = now() WHERE id = $3 RETURNING *",
    )
    .bind(&request.email)
    .bind(&request.name)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(map_user_write_error)?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Authentication required or invalid", body = crate::error::ErrorEnvelope),
        (status = 404, description = "User not found", body = crate::error::ErrorEnvelope)
    )
)]
pub(crate) async fn delete_user(
    State(state): State<AppState>,
    crate::auth::AuthUser(_user_id): crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn map_user_write_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(db_error) if db_error.is_unique_violation() => {
            AppError::Conflict("email already exists".into())
        }
        error => AppError::Database(error),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user).get(list_users))
        .route(
            "/users/{id}",
            get(get_user).patch(update_user).delete(delete_user),
        )
}
