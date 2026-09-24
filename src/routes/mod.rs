use axum::Router;

use crate::state::AppState;

pub mod auth;
pub mod cameras;
pub mod catalog;
pub mod health;
pub mod popularity;
pub mod route_misses;
pub mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(cameras::router())
        .merge(catalog::router())
        .merge(popularity::router())
        .merge(route_misses::router())
        .merge(users::router())
        .merge(auth::router())
}
