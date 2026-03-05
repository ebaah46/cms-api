use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc};

pub type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

/// Creates a rate limiter for authentication endpoints
/// Allows `requests_per_minute` requests per minute
pub fn create_auth_rate_limiter(requests_per_minute: u32) -> SharedRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap())
        .allow_burst(NonZeroU32::new(5).unwrap());

    Arc::new(RateLimiter::direct(quota))
}

/// Rate limit response when limit is exceeded
pub fn rate_limit_exceeded() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("Retry-After", "60")],
        "Too many requests. Please try again later.",
    )
        .into_response()
}
