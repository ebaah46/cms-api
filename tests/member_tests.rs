mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::TestApp;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_member() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{
            "first_name": "John",
            "last_name": "Doe",
            "email": "john.doe@example.com",
            "phone": "555-0100"
        }"#,
        ))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::CREATED, "Body: {}", body);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["first_name"], "John");
    assert_eq!(json["last_name"], "Doe");
    assert_eq!(json["email"], "john.doe@example.com");
    assert_eq!(json["membership_status"], "active");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_members() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a member first
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Jane","last_name":"Smith"}"#))
        .unwrap();
    app.request(req).await;

    // List members
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;
    dbg!(&body);
    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["data"].is_array());
    assert_eq!(json["total"], 1);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_get_member() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Bob","last_name":"Wilson"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let member_id = created["id"].as_str().unwrap();

    // Get the member
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/members/{}", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["first_name"], "Bob");
    assert_eq!(json["last_name"], "Wilson");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_member() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Alice","last_name":"Brown"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    dbg!(&created);
    let member_id = created["id"].as_str().unwrap();

    // Update the member
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/members/{}", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"first_name":"Alicia","email":"alicia@example.com"}"#,
        ))
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["first_name"], "Alicia");
    assert_eq!(json["last_name"], "Brown"); // unchanged
    assert_eq!(json["email"], "alicia@example.com");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_delete_member_soft_delete() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"first_name":"ToDelete","last_name":"Member"}"#,
        ))
        .unwrap();

    let (_, body) = app.request(req).await;
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let member_id = created["id"].as_str().unwrap();

    // Delete the member
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/members/{}", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.request(req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Try to get the deleted member - should return 404
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/members/{}", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.request(req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Verify soft delete - member still exists in DB
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM members WHERE id = $1::uuid AND deleted_at IS NOT NULL",
    )
    .bind(member_id)
    .fetch_one(&app.pool)
    .await
    .unwrap();

    assert_eq!(count.0, 1, "Member should be soft deleted");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_search_members() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create members
    for (first, last) in [("John", "Doe"), ("Jane", "Doe"), ("Bob", "Smith")] {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/members")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(format!(
                r#"{{"first_name":"{}","last_name":"{}"}}"#,
                first, last
            )))
            .unwrap();
        app.request(req).await;
    }

    // Search for "Doe"
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/members?search=Doe")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["total"], 2);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_member_validation() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Try to create member without required fields
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"OnlyFirst"}"#))
        .unwrap();

    let (status, _) = app.request(req).await;

    // Should fail validation (missing last_name)
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY);

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_get_member_detail() {
    let app = TestApp::new().await;
    app.cleanup().await;

    app.create_test_user("admin@example.com", "password123", "admin")
        .await;
    let token = app.login("admin@example.com", "password123").await;

    // Create a member
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/members")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"first_name":"Bob","last_name":"Wilson"}"#))
        .unwrap();

    let (_, body) = app.request(req).await;
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let member_id = created["id"].as_str().unwrap();

    // Get the member
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/members/{}", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["first_name"], "Bob");
    assert_eq!(json["last_name"], "Wilson");

    // Retrieve automatically created member detail
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/members/{}/detail", member_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app.request(req).await;
    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    dbg!(&json);
    assert_eq!(json["communicant"], false);
    assert_eq!(json["marital_status"], "single");
    assert_eq!(json["education_level"], "none");
    assert_eq!(json["place_of_birth"], serde_json::Value::Null);
    assert_eq!(json["spouse_name"], serde_json::Value::Null);

    app.cleanup().await;
}
