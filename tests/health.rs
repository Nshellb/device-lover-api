use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use device_lover_api::build_app;
use device_lover_api::config::Settings;
use device_lover_api::state::AppState;
use http_body_util::BodyExt;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

const UNUSED_DATABASE_URL: &str = "postgres://postgres:postgres@127.0.0.1/device_lover_test";

fn test_app(database_url: &str) -> Result<(axum::Router, PgPool), sqlx::Error> {
    let settings = Settings {
        host: [0, 0, 0, 0].into(),
        port: 0,
        database_url: database_url.to_owned(),
        jwt_secret: "test-jwt-secret".to_owned(),
    };
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(database_url)?;
    let state = AppState::new(settings, pool.clone());
    Ok((build_app(state, Duration::from_secs(5)), pool))
}

#[tokio::test]
async fn health_returns_ok_status() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping health check: DATABASE_URL is not set");
            return;
        }
    };

    let (app, pool) = match test_app(&database_url) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("skipping health check: invalid DATABASE_URL: {error}");
            return;
        }
    };

    if let Err(error) = pool.acquire().await {
        eprintln!("skipping health check: database is unreachable: {error}");
        return;
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["database"], "ok");
}

#[tokio::test]
async fn unknown_route_returns_json_404() {
    let (app, _) = test_app(UNUSED_DATABASE_URL).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "not_found");
}
