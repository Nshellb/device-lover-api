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
async fn openapi_spec_is_served() {
    let (app, _) = test_app(UNUSED_DATABASE_URL).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("openapi").is_some());
    assert!(json["paths"].as_object().unwrap().contains_key("/health"));
    for path in [
        "/api/v1/catalog/schema",
        "/api/v1/devices",
        "/api/v1/devices/{identifier}",
        "/api/v1/comparisons",
        "/api/v1/home",
        "/api/v1/admin/cameras",
        "/api/v1/cameras",
        "/api/v1/cameras/comparisons",
        "/api/v1/route-misses",
        "/api/v1/admin/route-misses",
        "/api/v1/admin/device-brands",
        "/api/v1/device-selections",
        "/api/v1/popular-devices",
    ] {
        assert!(json["paths"].as_object().unwrap().contains_key(path));
    }
    assert!(json["paths"]["/api/v1/admin/cameras"]["get"].is_object());
    for method in ["post", "put", "patch", "delete"] {
        assert!(json["paths"]["/api/v1/admin/cameras"].get(method).is_none());
    }
    assert!(
        json["components"]["schemas"]["CameraResponse"]["properties"]["releaseMonth"].is_object()
    );
}
