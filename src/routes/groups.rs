use axum::{
    Json, Router, debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::ListResponse;
use crate::dto::group_dto::{
    AddMemberToGroupRequest, CreateGroupRequest, GroupMemberResponse, GroupQuery, GroupResponse,
    UpdateGroupRequest,
};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route(
            "/:id",
            get(get_group).patch(update_group).delete(delete_group),
        )
        .route("/:id/members", get(get_group_members))
        .route(
            "/:id/members/:member_id",
            post(add_member_to_group).delete(remove_member_from_group),
        )
}

#[debug_handler]
async fn list_groups(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<GroupQuery>,
) -> Result<Json<ListResponse<GroupResponse>>, AppError> {
    let groups = state.group_service.find_all(query).await?;
    Ok(Json(groups))
}

#[debug_handler]
async fn create_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), AppError> {
    body.validate()?;
    let group = state.group_service.create(body).await?;
    Ok((StatusCode::CREATED, Json(group)))
}

#[debug_handler]
async fn get_group(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<GroupResponse>, AppError> {
    let group = state.group_service.find_by_id(id).await?;
    Ok(Json(group))
}

#[debug_handler]
async fn update_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    body.validate()?;
    let group = state.group_service.update(id, body).await?;
    Ok(Json(group))
}

#[debug_handler]
async fn delete_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.group_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
async fn get_group_members(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    let members = state.group_service.get_members(id).await?;
    Ok(Json(members))
}

#[debug_handler]
async fn add_member_to_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path((id, member_id)): Path<(Uuid, Uuid)>,
    AppJson(body): AppJson<AddMemberToGroupRequest>,
) -> Result<StatusCode, AppError> {
    body.validate()?;
    state
        .group_service
        .add_member(id, member_id, body.role.as_deref())
        .await?;
    Ok(StatusCode::CREATED)
}

#[debug_handler]
async fn remove_member_from_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path((id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state.group_service.remove_member(id, member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
