pub mod attendance;
pub mod auth;
pub mod groups;
pub mod households;
pub mod members;
pub mod services;
pub mod users;

use axum::Router;

use crate::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/households", households::router())
        .nest("/members", members::router())
        .nest("/groups", groups::router())
        .nest("/services", services::router())
        .nest("/attendance", attendance::router())
}
