use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use cms_api::{AppState, config::Config, routes};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::util::ServiceExt;

pub struct TestApp {
    pub pool: PgPool,
    pub app: Router,
    pub config: Config,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let config = Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/cms_api_test".to_string()
            }),
            jwt_secret: "test-secret-key-for-testing-only".to_string(),
            jwt_expiration_hours: 24,
            refresh_token_expiration_days: 7,
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            cors_origins: vec!["http://localhost:3000".to_string()],
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let state = AppState::new(pool.clone(), config.clone());

        let app = Router::new()
            .nest("/api/v1", routes::create_routes())
            .with_state(state);

        TestApp { pool, app, config }
    }

    pub async fn cleanup(&self) {
        // Clean up test data in reverse order of dependencies
        sqlx::query("DELETE FROM attendance")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM member_groups")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM refresh_tokens")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM members")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM households")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM groups")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM services")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM users")
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn request(&self, req: Request<Body>) -> (StatusCode, String) {
        let response = self.app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body).to_string();
        (status, body_str)
    }

    pub async fn create_test_user(&self, email: &str, password: &str, role: &str) -> String {
        use cms_api::services::auth_service::AuthService;

        let password_hash = AuthService::hash_password(password).unwrap();

        let user: (uuid::Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(email)
        .bind(&password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .unwrap();

        user.0.to_string()
    }

    pub async fn login(&self, email: &str, password: &str) -> String {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(format!(
                r#"{{"email":"{}","password":"{}"}}"#,
                email, password
            )))
            .unwrap();

        let (status, body) = self.request(req).await;
        assert_eq!(status, StatusCode::OK, "Login failed: {}", body);

        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        json["access_token"].as_str().unwrap().to_string()
    }
}
