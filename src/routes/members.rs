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
    CreateMemberRequest, ImportResult, MemberQuery, MemberResponse, UpdateMemberRequest,
};
use crate::dto::{ListResponse, PaginationParams};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};
use crate::repositories::group_repo::GroupRepository;
use crate::services::attendance_service::AttendanceService;
use crate::services::import_service::ImportService;
use crate::services::member_service::MemberService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_members).post(create_member))
        .route("/import", post(import_members))
        .layer(DefaultBodyLimit::max(2097152))
        .route(
            "/:id",
            get(get_member).patch(update_member).delete(delete_member),
        )
        .route("/:id/attendance", get(get_member_attendance))
        .route("/:id/groups", get(get_member_groups))
}

async fn list_members(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<MemberQuery>,
) -> Result<Json<ListResponse<MemberResponse>>, AppError> {
    let members = MemberService::find_all(&state.db, query).await?;
    Ok(Json(members))
}

async fn create_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), AppError> {
    body.validate()?;
    let member = MemberService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(member)))
}

async fn get_member(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<MemberResponse>, AppError> {
    let member = MemberService::find_by_id(&state.db, id).await?;
    Ok(Json(member))
}

async fn update_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateMemberRequest>,
) -> Result<Json<MemberResponse>, AppError> {
    body.validate()?;
    let member = MemberService::update(&state.db, id, body).await?;
    Ok(Json(member))
}

async fn delete_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    MemberService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_member_attendance(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<AttendanceResponse>>, AppError> {
    let attendance = AttendanceService::get_member_attendance(
        &state.db,
        id,
        Some(pagination.page()),
        Some(pagination.limit()),
    )
    .await?;
    Ok(Json(attendance))
}

async fn get_member_groups(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let groups = GroupRepository::get_member_groups(&state.db, id).await?;
    Ok(Json(groups.into_iter().map(|g| g.into()).collect()))
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

            let result = ImportService::import_members_from_csv(&state.db, &data).await?;
            return Ok(Json(result));
        }
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}
