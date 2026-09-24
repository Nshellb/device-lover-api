use std::collections::HashSet;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use device_lover_api::build_app;
use device_lover_api::config::Settings;
use device_lover_api::state::AppState;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

const UNUSED_DATABASE_URL: &str = "postgres://postgres:postgres@127.0.0.1/device_lover_test";

fn test_app(database_url: &str) -> (Router, PgPool) {
    let settings = Settings {
        host: [0, 0, 0, 0].into(),
        port: 0,
        database_url: database_url.to_owned(),
        jwt_secret: "test-jwt-secret".to_owned(),
    };
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(database_url)
        .expect("valid DATABASE_URL");
    let state = AppState::new(settings, pool.clone());
    (build_app(state, Duration::from_secs(5)), pool)
}

async fn send(app: &Router, method: Method, uri: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn response_json(response: Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

async fn get_json(app: &Router, uri: &str) -> Value {
    let response = send(app, Method::GET, uri).await;
    assert_eq!(response.status(), StatusCode::OK, "{uri}");
    response_json(response).await
}

fn with_query(pairs: &[(&str, &str)]) -> String {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs.iter().copied())
        .finish();
    format!("/api/v1/admin/cameras?{query}")
}

#[tokio::test]
async fn camera_query_validation_happens_before_database_access() {
    let (app, _) = test_app(UNUSED_DATABASE_URL);
    for query in [
        "unknown=1",
        "q=5d&q=6d",
        "series=EOS+5D&series=EOS+6D",
        "page=1&page=2",
        "page_size=10&page_size=20",
        "page=0",
        "page=10001",
        "page=-1",
        "page=1.5",
        "page=",
        "page=%2B1",
        "page_size=0",
        "page_size=101",
        "page_size=abc",
        "series=EOS+7D",
        "series=",
        "q=%",
        "q=%2",
        "q=%GG",
        "q=%00",
    ] {
        let uri = format!("/api/v1/admin/cameras?{query}");
        let response = send(&app, Method::GET, &uri).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(
            response_json(response).await["error"]["code"],
            "validation_error",
            "{uri}"
        );
    }
    let uri = with_query(&[("q", &"가".repeat(101))]);
    assert_eq!(
        send(&app, Method::GET, &uri).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn cameras_only_expose_read_endpoints() {
    let (app, _) = test_app(UNUSED_DATABASE_URL);
    for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        for uri in ["/api/v1/admin/cameras", "/api/v1/cameras"] {
            assert_eq!(
                send(&app, method.clone(), uri).await.status(),
                StatusCode::METHOD_NOT_ALLOWED
            );
        }
    }
    assert_eq!(
        send(&app, Method::GET, "/api/v1/admin/cameras/canon-eos-5d")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn camera_seed_filtering_and_pagination_work_with_database() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping camera database test: DATABASE_URL is not set");
        return;
    };
    let (app, pool) = test_app(&database_url);
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("camera schema and seed migrations should succeed");

    let all = get_json(&app, "/api/v1/admin/cameras").await;
    assert_eq!(get_json(&app, "/api/v1/cameras").await, all);
    let items = all["items"].as_array().unwrap();
    assert_eq!(
        all["pagination"],
        json!({"page": 1, "pageSize": 30, "total": 17, "totalPages": 1})
    );
    sqlx::raw_sql(include_str!("../migrations/0006_seed_canon_cameras.sql"))
        .execute(&pool)
        .await
        .expect("replaying camera seed should be idempotent");
    assert_eq!(get_json(&app, "/api/v1/admin/cameras").await, all);
    let names: HashSet<_> = items
        .iter()
        .map(|camera| camera["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        HashSet::from([
            "EOS 5D",
            "EOS 5D Mark II",
            "EOS 5D Mark III",
            "EOS 5D Mark IV",
            "EOS 5DS",
            "EOS 5DS R",
            "EOS 6D",
            "EOS 6D Mark II",
            "EOS 10D",
            "EOS 20D",
            "EOS 30D",
            "EOS 40D",
            "EOS 50D",
            "EOS 60D",
            "EOS 70D",
            "EOS 80D",
            "EOS 90D",
        ])
    );
    for camera in items {
        assert_eq!(camera["category"], "camera");
        assert_eq!(camera["brandSlug"], "canon");
        assert_eq!(camera["cameraType"], "DSLR");
        assert!(camera["effectiveMegapixels"].as_f64().unwrap() > 0.0);
        assert!(camera["bodyWeightG"].as_i64().unwrap() > 0);
        let source_url = camera["sourceUrl"].as_str().unwrap();
        assert!(
            source_url.starts_with("https://global.canon/en/c-museum/")
                || source_url.starts_with("https://global.canon/ja/c-museum/")
        );
        assert_eq!(camera["releaseMonth"].as_str().unwrap().len(), 7);
        for field in ["checkedAt", "createdAt", "updatedAt"] {
            chrono::DateTime::parse_from_rfc3339(camera[field].as_str().unwrap()).unwrap();
        }
    }

    // The same stable ordering must survive page boundaries.
    let mut paged_ids = Vec::new();
    for page in 1..=6 {
        let response = get_json(
            &app,
            &format!("/api/v1/admin/cameras?page={page}&page_size=3"),
        )
        .await;
        assert_eq!(response["pagination"]["total"], 17);
        assert_eq!(response["pagination"]["totalPages"], 6);
        paged_ids.extend(
            response["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|camera| camera["id"].clone()),
        );
    }
    assert_eq!(
        paged_ids,
        items
            .iter()
            .map(|camera| camera["id"].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(items.first().unwrap()["name"], "EOS 90D");
    assert_eq!(items.last().unwrap()["name"], "EOS 10D");
    assert_eq!(items.first().unwrap()["maxContinuousFps"], 11.0);
    assert!(
        items.first().unwrap()["continuousShootingNote"]
            .as_str()
            .is_some_and(|note| !note.is_empty())
    );

    for (series, expected) in [("EOS 5D", 6), ("EOS 6D", 2), ("EOS x0D", 9)] {
        let filtered = get_json(&app, &with_query(&[("series", series)])).await;
        assert_eq!(filtered["pagination"]["total"], expected);
        assert!(
            filtered["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|camera| camera["series"] == series)
        );
    }
    for q in ["Canon 5d", "캐논 5D", "CANON eos ５Ｄ"] {
        assert_eq!(
            get_json(&app, &with_query(&[("q", q)])).await["pagination"]["total"],
            6
        );
    }
    let filtered = get_json(&app, &with_query(&[("q", "mark ii"), ("series", "EOS 6D")])).await;
    assert_eq!(filtered["pagination"]["total"], 1);
    assert_eq!(filtered["items"][0]["name"], "EOS 6D Mark II");
    for q in ["nonexistent-model", "%", "_"] {
        let empty = get_json(&app, &with_query(&[("q", q)])).await;
        assert_eq!(empty["items"], json!([]));
        assert_eq!(empty["pagination"]["total"], 0);
        assert_eq!(empty["pagination"]["totalPages"], 0);
    }
    let beyond = get_json(&app, "/api/v1/admin/cameras?page=10000&page_size=100").await;
    assert_eq!(beyond["items"], json!([]));
    assert_eq!(beyond["pagination"]["total"], 17);

    let comparison = get_json(
        &app,
        "/api/v1/cameras/comparisons?identifiers=canon-eos-90d&identifiers=canon-eos-5d",
    )
    .await;
    assert_eq!(
        comparison["canonicalPath"],
        "/canon-eos-90d-vs-canon-eos-5d"
    );
    assert_eq!(comparison["schemaVersion"], 1);
    assert_eq!(comparison["devices"][0]["name"], "EOS 90D");
    assert_eq!(comparison["devices"][1]["name"], "EOS 5D");

    for uri in [
        "/api/v1/cameras/comparisons?identifiers=canon-eos-90d&identifiers=canon-eos-90d",
        "/api/v1/cameras/comparisons?identifiers=canon-eos-90d&identifiers=galaxy-s24",
    ] {
        let response = send(&app, Method::GET, uri).await;
        assert!(
            matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND
            ),
            "{uri}"
        );
    }

    // Adding camera rows must not publish them in the existing device catalog.
    let public = get_json(&app, "/api/v1/devices").await;
    assert!(
        public["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|device| device["brandSlug"] != "canon")
    );
    pool.close().await;
}
