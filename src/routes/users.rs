use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::dto::{ListResponse, PaginationParams};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::RequireAdmin;
use crate::services::user_service::UserService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user).patch(update_user).delete(delete_user))
}

async fn list_users(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>, AppError> {
    let users = UserService::find_all(&state.db, pagination).await?;
    Ok(Json(users))
}

async fn create_user(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    body.validate()?;
    let user = UserService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn get_user(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = UserService::find_by_id(&state.db, id).await?;
    Ok(Json(user))
}

async fn update_user(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    body.validate()?;
    let user = UserService::update(&state.db, id, body).await?;
    Ok(Json(user))
}

async fn delete_user(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    UserService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
