use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{
    Method, Request, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use axum::response::Response;
use device_lover_api::build_app;
use device_lover_api::config::Settings;
use device_lover_api::state::AppState;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "test-jwt-secret";
const PASSWORD: &str = "hunter22";

fn test_app(database_url: &str) -> Result<(Router, PgPool), sqlx::Error> {
    let settings = Settings {
        host: [0, 0, 0, 0].into(),
        port: 0,
        database_url: database_url.to_owned(),
        jwt_secret: JWT_SECRET.to_owned(),
    };
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(database_url)?;
    let state = AppState::new(settings, pool.clone());

    Ok((build_app(state, Duration::from_secs(5)), pool))
}

async fn connected_test_app(database_url: &str) -> Option<Router> {
    let (app, pool) = match test_app(database_url) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("skipping auth test: invalid DATABASE_URL: {error}");
            return None;
        }
    };

    if let Err(error) = pool.acquire().await {
        eprintln!("skipping auth test: database is unreachable: {error}");
        return None;
    }

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("auth migrations should succeed");

    Some(app)
}

async fn send_json(app: &Router, method: Method, uri: &str, payload: Value) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send_json_with_authorization(
    app: &Router,
    method: Method,
    uri: &str,
    authorization: &str,
    payload: Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, authorization)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send_empty(app: &Router, method: Method, uri: &str) -> Response {
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

async fn send_empty_with_authorization(
    app: &Router,
    method: Method,
    uri: &str,
    authorization: &str,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(AUTHORIZATION, authorization)
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

fn unique_email(label: &str) -> String {
    format!("{label}-{}@example.com", Uuid::new_v4())
}

async fn create_user(app: &Router, email: &str) -> Value {
    let response = send_json(
        app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "Auth Test", "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    response_json(response).await
}

async fn login_token(app: &Router, email: &str) -> String {
    let response = send_json(
        app,
        Method::POST,
        "/auth/login",
        json!({ "email": email, "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    response_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn login_with_correct_credentials_returns_token() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("login-success");
    create_user(&app, &email).await;

    let response = send_json(
        &app,
        Method::POST,
        "/auth/login",
        json!({ "email": email, "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert!(!body["token"].as_str().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn login_with_wrong_password_returns_unauthorized() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("login-wrong-password");
    create_user(&app, &email).await;

    let response = send_json(
        &app,
        Method::POST,
        "/auth/login",
        json!({ "email": email, "password": "wrong-password" }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "unauthorized");
}

#[tokio::test]
async fn login_with_unknown_email_returns_unauthorized() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };

    let response = send_json(
        &app,
        Method::POST,
        "/auth/login",
        json!({ "email": unique_email("unknown"), "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "unauthorized");
}

#[tokio::test]
async fn protected_user_mutations_reject_missing_authorization() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let id = Uuid::new_v4();
    let uri = format!("/users/{id}");

    let patch_response = send_json(&app, Method::PATCH, &uri, json!({ "name": "Blocked" })).await;
    assert_eq!(patch_response.status(), StatusCode::UNAUTHORIZED);

    let delete_response = send_empty(&app, Method::DELETE, &uri).await;
    assert_eq!(delete_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_user_mutations_reject_invalid_token() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let id = Uuid::new_v4();
    let uri = format!("/users/{id}");

    let patch_response = send_json_with_authorization(
        &app,
        Method::PATCH,
        &uri,
        "Bearer not-a-real-token",
        json!({ "name": "Blocked" }),
    )
    .await;
    assert_eq!(patch_response.status(), StatusCode::UNAUTHORIZED);

    let delete_response =
        send_empty_with_authorization(&app, Method::DELETE, &uri, "Bearer not-a-real-token").await;
    assert_eq!(delete_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_user_mutations_accept_valid_token() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping auth test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("valid-token");
    let created = create_user(&app, &email).await;
    let id = created["id"].as_str().unwrap();
    let uri = format!("/users/{id}");
    let token = login_token(&app, &email).await;
    let authorization = format!("Bearer {token}");

    let patch_response = send_json_with_authorization(
        &app,
        Method::PATCH,
        &uri,
        &authorization,
        json!({ "name": "Updated" }),
    )
    .await;
    assert_eq!(patch_response.status(), StatusCode::OK);

    let delete_response =
        send_empty_with_authorization(&app, Method::DELETE, &uri, &authorization).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}
