use axum::body::Body;
use axum::http::Request;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::post,
};
use validator::Validate;

use crate::AppState;
use crate::dto::auth_dto::{
    LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, SetupRequest, SetupResponse,
};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::rate_limit::{
    SharedRateLimiter, create_auth_rate_limiter, rate_limit_exceeded,
};

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

    let response = state
        .auth_service
        .login(&state.config, &body.email, &body.password)
        .await?;

    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let response = state
        .auth_service
        .refresh(&state.config, &body.refresh_token)
        .await?;

    Ok(Json(response))
}

async fn logout(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshRequest>,
) -> Result<StatusCode, AppError> {
    state.auth_service.logout(&body.refresh_token).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// One-time setup endpoint to create the initial admin user.
/// Only works when no users exist in the database.
async fn setup(
    State(state): State<AppState>,
    AppJson(body): AppJson<SetupRequest>,
) -> Result<(StatusCode, Json<SetupResponse>), AppError> {
    body.validate()?;
    let response = state.user_service.setup(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
