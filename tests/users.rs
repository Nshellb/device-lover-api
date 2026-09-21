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
            eprintln!("skipping user test: invalid DATABASE_URL: {error}");
            return None;
        }
    };

    if let Err(error) = pool.acquire().await {
        eprintln!("skipping user test: database is unreachable: {error}");
        return None;
    }

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("user migrations should succeed");

    Some(app)
}

async fn send_json(app: &Router, method: Method, uri: &str, payload: Value) -> Response {
    let payload = payload.to_string();
    send_raw_json(app, method, uri, &payload).await
}

async fn send_raw_json(app: &Router, method: Method, uri: &str, payload: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send_json_with_auth(
    app: &Router,
    method: Method,
    uri: &str,
    token: &str,
    payload: Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, format!("Bearer {token}"))
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

async fn send_empty_with_auth(app: &Router, method: Method, uri: &str, token: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(AUTHORIZATION, format!("Bearer {token}"))
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

async fn login(app: &Router, email: &str, password: &str) -> String {
    let response = send_json(
        app,
        Method::POST,
        "/auth/login",
        json!({ "email": email, "password": password }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    response_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn create_and_get_user() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("create-get");

    let create_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "Ada", "password": PASSWORD }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = response_json(create_response).await;
    assert_eq!(created["email"], email);
    assert_eq!(created["name"], "Ada");
    let id = created["id"].as_str().unwrap();

    let get_response = send_empty(&app, Method::GET, &format!("/users/{id}")).await;
    assert_eq!(get_response.status(), StatusCode::OK);
    let fetched = response_json(get_response).await;
    assert_eq!(fetched["id"], created["id"]);
    assert_eq!(fetched["email"], email);
    assert_eq!(fetched["name"], "Ada");
}

#[tokio::test]
async fn list_includes_created_user() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("list");

    let create_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "Grace", "password": PASSWORD }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let list_response = send_empty(&app, Method::GET, "/users").await;
    assert_eq!(list_response.status(), StatusCode::OK);
    let users = response_json(list_response).await;
    let contains_user = users
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["email"] == email && user["name"] == "Grace");
    assert!(contains_user);
}

#[tokio::test]
async fn update_changes_user_fields() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let original_email = unique_email("update-original");
    let updated_email = unique_email("update-new");

    let create_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": original_email, "name": "Before", "password": PASSWORD }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = response_json(create_response).await;
    let id = created["id"].as_str().unwrap();

    let token = login(&app, &original_email, PASSWORD).await;
    let update_response = send_json_with_auth(
        &app,
        Method::PATCH,
        &format!("/users/{id}"),
        &token,
        json!({ "email": updated_email, "name": "After" }),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = response_json(update_response).await;
    assert_eq!(updated["id"], created["id"]);
    assert_eq!(updated["email"], updated_email);
    assert_eq!(updated["name"], "After");
}

#[tokio::test]
async fn patch_with_empty_body_leaves_user_unchanged() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("empty-patch");
    let name = "Stay Put";

    let create_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": name, "password": PASSWORD }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = response_json(create_response).await;
    let id = created["id"].as_str().unwrap();

    let token = login(&app, &email, PASSWORD).await;
    let update_response = send_json_with_auth(
        &app,
        Method::PATCH,
        &format!("/users/{id}"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = response_json(update_response).await;
    assert_eq!(updated["email"], email);
    assert_eq!(updated["name"], name);
}

#[tokio::test]
async fn delete_removes_user() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("delete");

    let create_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "Delete Me", "password": PASSWORD }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = response_json(create_response).await;
    let id = created["id"].as_str().unwrap();

    let token = login(&app, &email, PASSWORD).await;
    let delete_response =
        send_empty_with_auth(&app, Method::DELETE, &format!("/users/{id}"), &token).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_response = send_empty(&app, Method::GET, &format!("/users/{id}")).await;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_email_returns_bad_request() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };

    let response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": "invalid-email", "name": "Invalid", "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn malformed_json_returns_bad_request_envelope() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };

    let response = send_raw_json(&app, Method::POST, "/users", "{not json").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn missing_required_field_returns_bad_request_envelope() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };

    let response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": unique_email("missing-field"), "password": PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn duplicate_email_returns_conflict() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let email = unique_email("duplicate");

    let first_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "First", "password": PASSWORD }),
    )
    .await;
    assert_eq!(first_response.status(), StatusCode::CREATED);

    let second_response = send_json(
        &app,
        Method::POST,
        "/users",
        json!({ "email": email, "name": "Second", "password": PASSWORD }),
    )
    .await;
    assert_eq!(second_response.status(), StatusCode::CONFLICT);
    let body = response_json(second_response).await;
    assert_eq!(body["error"]["code"], "conflict");
    assert_eq!(body["error"]["message"], "email already exists");
}

#[tokio::test]
async fn nonexistent_user_returns_not_found() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("skipping user test: DATABASE_URL is not set");
            return;
        }
    };
    let Some(app) = connected_test_app(&database_url).await else {
        return;
    };
    let id = Uuid::new_v4();

    let response = send_empty(&app, Method::GET, &format!("/users/{id}")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
