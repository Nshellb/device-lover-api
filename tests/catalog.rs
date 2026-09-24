use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use device_lover_api::build_app;
use device_lover_api::config::Settings;
use device_lover_api::state::AppState;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

const UNUSED_DATABASE_URL: &str = "postgres://postgres:postgres@127.0.0.1/device_lover_test";

fn test_app(database_url: &str) -> Result<(Router, PgPool), sqlx::Error> {
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

async fn get(app: &Router, uri: &str) -> Response {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn response_json(response: Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn validation_errors_do_not_require_database_access() {
    let (app, _) = test_app(UNUSED_DATABASE_URL).unwrap();

    for uri in [
        "/api/v1/devices?unknown=1",
        "/api/v1/devices?category=camera",
        "/api/v1/admin/devices?category=camera",
        "/api/v1/admin/devices?search_field=slug",
        "/api/v1/admin/devices?publication_status=private",
        "/api/v1/admin/devices?sort=updated_at_desc",
        "/api/v1/admin/devices?brand=Samsung",
        "/api/v1/admin/device-brands?unexpected=1",
        "/api/v1/devices?page=0",
        "/api/v1/devices?page=1&page=2",
        "/api/v1/comparisons",
        "/api/v1/comparisons?identifiers=a&identifiers=b&identifiers=c&identifiers=d",
        "/api/v1/home?unexpected=1",
        "/api/v1/catalog/schema?unexpected=1",
    ] {
        let response = get(&app, uri).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let body = response_json(response).await;
        assert_eq!(body["error"]["code"], "validation_error", "{uri}");
    }
}

#[tokio::test]
async fn catalog_migration_and_empty_responses_work_with_database() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping catalog database test: DATABASE_URL is not set");
            return;
        }
    };
    let (app, pool) = match test_app(&database_url) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("skipping catalog database test: invalid DATABASE_URL: {error}");
            return;
        }
    };
    if let Err(error) = pool.acquire().await {
        eprintln!("skipping catalog database test: database is unreachable: {error}");
        return;
    }
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("catalog migration should succeed");

    let schema_response = get(&app, "/api/v1/catalog/schema").await;
    assert_eq!(schema_response.status(), StatusCode::OK);
    let schema = response_json(schema_response).await;
    assert_eq!(schema["schemaVersion"], 1);
    assert_eq!(schema["maxComparisonDevices"], 3);
    assert_eq!(schema["sections"].as_array().unwrap().len(), 6);

    let list_response = get(&app, "/api/v1/devices?page_size=10").await;
    assert_eq!(list_response.status(), StatusCode::OK);
    let list = response_json(list_response).await;
    assert!(list["items"].is_array());
    assert_eq!(list["pagination"]["page"], 1);
    assert_eq!(list["pagination"]["pageSize"], 10);

    let home_response = get(&app, "/api/v1/home").await;
    assert_eq!(home_response.status(), StatusCode::OK);
    let home = response_json(home_response).await;
    assert_eq!(home["schemaVersion"], 1);
    assert!(home["devices"].as_array().unwrap().len() <= 2);
}
