mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use common::TestApp;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_household() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin").await;
    let token = app.login("admin@example.com", "password123").await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/households")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{
            "name": "The Smiths",
            "address": "123 Main St",
            "phone": "555-0100"
        }"#))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::CREATED);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["name"], "The Smiths");
    assert_eq!(json["address"], "123 Main St");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_link_member_to_household() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin").await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a household
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/households")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"name":"The Johnsons"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let household: serde_json::Value = serde_json::from_str(&body).unwrap();
    let household_id = household["id"].as_str().unwrap();

    // Create a member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Mike","last_name":"Johnson"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let member: serde_json::Value = serde_json::from_str(&body).unwrap();
    let member_id = member["id"].as_str().unwrap();

    // Link member to household
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/households/{}/members/{}", household_id, member_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"household_role":"head"}"#))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["household_id"], household_id);
    assert_eq!(json["household_role"], "head");

    // Get household members
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/households/{}/members", household_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let members: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["first_name"], "Mike");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_unlink_member_from_household() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin").await;
    let token = app.login("admin@example.com", "password123").await;

    // Create household and member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/households")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"name":"The Williams"}"#))
        .unwrap();
    let (_, body) = app.request(req).await;
    let household: serde_json::Value = serde_json::from_str(&body).unwrap();
    let household_id = household["id"].as_str().unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Sarah","last_name":"Williams"}"#))
        .unwrap();
    let (_, body) = app.request(req).await;
    let member: serde_json::Value = serde_json::from_str(&body).unwrap();
    let member_id = member["id"].as_str().unwrap();

    // Link
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/households/{}/members/{}", household_id, member_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{}"#))
        .unwrap();
    app.request(req).await;

    // Unlink
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/households/{}/members/{}", household_id, member_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["household_id"].is_null());

    app.cleanup().await;
}
