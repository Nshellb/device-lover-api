use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::User;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_email(&self.email)?;
        validate_name(&self.name)?;

        if self.password.chars().count() < 8 {
            return Err(AppError::Validation(
                "password must be at least 8 characters".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub name: Option<String>,
}

impl UpdateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(email) = &self.email {
            validate_email(email)?;
        }

        if let Some(name) = &self.name {
            validate_name(name)?;
        }

        Ok(())
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if email.is_empty() {
        return Err(AppError::Validation("email must not be empty".into()));
    }

    if !email.contains('@') {
        return Err(AppError::Validation("email must contain '@'".into()));
    }

    if email.chars().any(char::is_whitespace) {
        return Err(AppError::Validation(
            "email must not contain whitespace".into(),
        ));
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }

    Ok(())
}
