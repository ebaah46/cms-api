use axum::{
    Router,
    http::{Method, header},
};
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cms_api::{
    AppStateBuilder, RepositoryManager,
    config::Config,
    routes,
    services::{
        attendance_service::AttendanceService, auth_service::AuthService,
        group_service::GroupService, household_service::HouseholdService,
        import_service::ImportService, member_service::CachedMemberService,
        service_service::ServiceService, user_service::CachedUserService,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cms_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");

    // Create repositories needed in backend
    let repo_manager = RepositoryManager::new(pool);
    let attendance_repo = repo_manager.get_attendance_repo();
    let member_repo = repo_manager.get_member_repo();
    let service_repo = repo_manager.get_service_repo();
    let household_repo = repo_manager.get_household_repo();
    let group_repo = repo_manager.get_group_repo();
    let refresh_token_repo = repo_manager.get_refresh_token_repo();
    let user_repo = repo_manager.get_user_repo();

    // Create app state
    let state = AppStateBuilder::new(config.clone())
        .attendance_service(Arc::new(AttendanceService::new(
            attendance_repo.clone(),
            member_repo.clone(),
            service_repo.clone(),
        )))
        .member_service(Arc::new(CachedMemberService::new(member_repo.clone())))
        .auth_service(Arc::new(AuthService::new(
            user_repo.clone(),
            refresh_token_repo.clone(),
        )))
        .group_service(Arc::new(GroupService::new(
            group_repo.clone(),
            member_repo.clone(),
        )))
        .household_service(Arc::new(HouseholdService::new(
            household_repo.clone(),
            member_repo.clone(),
        )))
        .service_service(Arc::new(ServiceService::new(service_repo.clone())))
        .import_service(Arc::new(ImportService::new(member_repo.clone())))
        .user_service(Arc::new(CachedUserService::new(user_repo.clone())))
        .build();

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .cors_origins
                .iter()
                .filter_map(|origin| origin.parse().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true);

    // Build router
    let app = Router::new()
        .nest("/api/v1", routes::create_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
