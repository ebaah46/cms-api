use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::dto::group_dto::{
    AddMemberToGroupRequest, CreateGroupRequest, GroupMemberResponse, GroupQuery, GroupResponse,
    UpdateGroupRequest,
};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};
use crate::services::group_service::GroupService;
use crate::AppState;

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
    let groups = GroupService::find_all(&state.db, query).await?;
    Ok(Json(groups))
}

#[debug_handler]
async fn create_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), AppError> {
    body.validate()?;
    let group = GroupService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(group)))
}

#[debug_handler]
async fn get_group(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<GroupResponse>, AppError> {
    let group = GroupService::find_by_id(&state.db, id).await?;
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
    let group = GroupService::update(&state.db, id, body).await?;
    Ok(Json(group))
}

#[debug_handler]
async fn delete_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    GroupService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
async fn get_group_members(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    let members = GroupService::get_members(&state.db, id).await?;
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
    GroupService::add_member(&state.db, id, member_id, body.role.as_deref()).await?;
    Ok(StatusCode::CREATED)
}

#[debug_handler]
async fn remove_member_from_group(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path((id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    GroupService::remove_member(&state.db, id, member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
