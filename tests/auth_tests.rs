mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use common::TestApp;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_login_success() {
    let app = TestApp::new().await;
    app.cleanup().await;

    // Create a test user
    app.create_test_user("test@example.com", "password123", "staff").await;

    // Login
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"test@example.com","password":"password123"}"#))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["access_token"].is_string());
    assert!(json["refresh_token"].is_string());
    assert_eq!(json["token_type"], "Bearer");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_login_invalid_credentials() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("test@example.com", "password123", "staff").await;

    // Try login with wrong password
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"test@example.com","password":"wrongpassword"}"#))
        .unwrap();

    let (status, _) = app.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_login_nonexistent_user() {
    let app = TestApp::new().await;
    app.cleanup().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"nonexistent@example.com","password":"password123"}"#))
        .unwrap();

    let (status, _) = app.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_refresh_token() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("test@example.com", "password123", "staff").await;

    // Login to get tokens
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"test@example.com","password":"password123"}"#))
        .unwrap();

    let (status, body) = app.request(req).await;
    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let refresh_token = json["refresh_token"].as_str().unwrap();

    // Use refresh token
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("Content-Type", "application/json")
        .body(Body::from(format!(r#"{{"refresh_token":"{}"}}"#, refresh_token)))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["access_token"].is_string());

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_logout() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("test@example.com", "password123", "staff").await;

    // Login
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"test@example.com","password":"password123"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let refresh_token = json["refresh_token"].as_str().unwrap();

    // Logout
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("Content-Type", "application/json")
        .body(Body::from(format!(r#"{{"refresh_token":"{}"}}"#, refresh_token)))
        .unwrap();

    let (status, _) = app.request(req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Try to use the refresh token again - should fail
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("Content-Type", "application/json")
        .body(Body::from(format!(r#"{{"refresh_token":"{}"}}"#, refresh_token)))
        .unwrap();

    let (status, _) = app.request(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_protected_route_without_token() {
    let app = TestApp::new().await;
    app.cleanup().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/members")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_protected_route_with_valid_token() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("test@example.com", "password123", "staff").await;
    let token = app.login("test@example.com", "password123").await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    app.cleanup().await;
}
