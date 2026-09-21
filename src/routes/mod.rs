use axum::Router;

use crate::state::AppState;

pub mod auth;
pub mod health;
pub mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(users::router())
        .merge(auth::router())
}
