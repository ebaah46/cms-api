use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::attendance_dto::AttendanceResponse;
use crate::dto::group_dto::GroupResponse;
use crate::dto::member_dto::{
    CreateMemberRequest, ImportResult, MemberDetailResponse, MemberQuery, MemberResponse,
    UpdateMemberDetailRequest, UpdateMemberRequest,
};
use crate::dto::{ListResponse, PaginationParams};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_members).post(create_member))
        .route("/import", post(import_members))
        .layer(DefaultBodyLimit::max(2097152))
        .route(
            "/:id",
            get(get_member).patch(update_member).delete(delete_member),
        )
        .route(
            "/:id/detail",
            get(get_member_detail).patch(update_member_detail),
        )
        .route("/:id/attendance", get(get_member_attendance))
        .route("/:id/groups", get(get_member_groups))
}

async fn list_members(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<MemberQuery>,
) -> Result<Json<ListResponse<MemberResponse>>, AppError> {
    let members = state.member_service.find_all(query).await?;
    Ok(Json(members))
}

async fn create_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), AppError> {
    body.validate()?;
    let member = state.member_service.create(body).await?;
    Ok((StatusCode::CREATED, Json(member)))
}

async fn get_member(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<MemberResponse>, AppError> {
    let member = state.member_service.find_by_id(id).await?;
    Ok(Json(member))
}

async fn get_member_detail(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<MemberDetailResponse>, AppError> {
    let member = state.member_service.find_detail_by_id(id).await?;
    Ok(Json(member))
}

async fn update_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateMemberRequest>,
) -> Result<Json<MemberResponse>, AppError> {
    body.validate()?;
    let member = state.member_service.update(id, body).await?;
    Ok(Json(member))
}

async fn update_member_detail(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateMemberDetailRequest>,
) -> Result<Json<MemberDetailResponse>, AppError> {
    body.validate()?;
    let member = state.member_service.update_detail(id, body).await?;
    Ok(Json(member))
}

async fn delete_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.member_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_member_attendance(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<AttendanceResponse>>, AppError> {
    let attendance = state
        .attendance_service
        .get_member_attendance(id, Some(pagination.page()), Some(pagination.limit()))
        .await?;
    Ok(Json(attendance))
}

async fn get_member_groups(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let groups = state.group_service.get_member_groups(id).await?;
    Ok(Json(groups))
}

async fn import_members(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?;

            let result = state.import_service.import_members_from_csv(&data).await?;
            return Ok(Json(result));
        }
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}
