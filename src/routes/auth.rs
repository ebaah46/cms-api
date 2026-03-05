use axum::{
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::post,
    Json, Router,
};
use axum::http::Request;
use axum::body::Body;
use validator::Validate;

use crate::dto::auth_dto::{
    LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, SetupRequest, SetupResponse,
};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::rate_limit::{create_auth_rate_limiter, rate_limit_exceeded, SharedRateLimiter};
use crate::services::auth_service::AuthService;
use crate::services::user_service::UserService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    // Create rate limiter: 10 login attempts per minute
    let login_limiter = create_auth_rate_limiter(10);

    Router::new()
        .route("/login", post(login))
        .route_layer(middleware::from_fn_with_state(
            login_limiter,
            rate_limit_middleware,
        ))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/setup", post(setup))
}

async fn rate_limit_middleware(
    State(limiter): State<SharedRateLimiter>,
    request: Request<Body>,
    next: Next,
) -> Response {
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_) => rate_limit_exceeded(),
    }
}

async fn login(
    State(state): State<AppState>,
    AppJson(body): AppJson<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    body.validate()?;

    let response = AuthService::login(&state.db, &state.config, &body.email, &body.password).await?;

    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let response = AuthService::refresh(&state.db, &state.config, &body.refresh_token).await?;

    Ok(Json(response))
}

async fn logout(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshRequest>,
) -> Result<StatusCode, AppError> {
    AuthService::logout(&state.db, &body.refresh_token).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// One-time setup endpoint to create the initial admin user.
/// Only works when no users exist in the database.
async fn setup(
    State(state): State<AppState>,
    AppJson(body): AppJson<SetupRequest>,
) -> Result<(StatusCode, Json<SetupResponse>), AppError> {
    body.validate()?;
    let response = UserService::setup(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
