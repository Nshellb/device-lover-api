use axum::Router;

use crate::state::AppState;

pub mod auth;
pub mod brands;
pub mod cameras;
pub mod catalog;
pub mod health;
pub mod popularity;
pub mod route_misses;
pub mod users;
pub mod software_versions;
pub mod wireless_technologies;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(brands::router())
        .merge(cameras::router())
        .merge(catalog::router())
        .merge(popularity::router())
        .merge(route_misses::router())
        .merge(users::router())
        .merge(software_versions::router())
        .merge(wireless_technologies::router())
        .merge(auth::router())
}
