#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health,
        crate::routes::users::create_user,
        crate::routes::users::list_users,
        crate::routes::users::get_user,
        crate::routes::users::update_user,
        crate::routes::users::delete_user,
        crate::routes::auth::login,
    ),
    components(schemas(
        crate::routes::health::HealthResponse,
        crate::dto::CreateUserRequest,
        crate::dto::UpdateUserRequest,
        crate::dto::UserResponse,
        crate::dto::LoginRequest,
        crate::dto::TokenResponse,
        crate::error::ErrorEnvelope,
    )),
    tags(
        (name = "health", description = "Service health"),
        (name = "users", description = "User management"),
        (name = "auth", description = "Authentication"),
    )
)]
pub struct ApiDoc;
