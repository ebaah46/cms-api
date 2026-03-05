use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, post},
};
use uuid::Uuid;

use crate::AppState;
use crate::dto::attendance_dto::{
    AttendanceResponse, BulkCheckInRequest, BulkCheckInResponse, CheckInRequest,
};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};
use crate::services::attendance_service::AttendanceService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(check_in))
        .route("/bulk", post(bulk_check_in))
        .route("/:id", delete(delete_attendance))
}

#[debug_handler]
async fn check_in(
    State(state): State<AppState>,
    user: RequireStaff,
    AppJson(body): AppJson<CheckInRequest>,
) -> Result<(StatusCode, Json<AttendanceResponse>), AppError> {
    let attendance =
        AttendanceService::check_in(&state.db, body.member_id, body.service_id, Some(user.0.id))
            .await?;
    Ok((StatusCode::CREATED, Json(attendance)))
}

#[debug_handler]
async fn bulk_check_in(
    State(state): State<AppState>,
    user: RequireStaff,
    AppJson(body): AppJson<BulkCheckInRequest>,
) -> Result<Json<BulkCheckInResponse>, AppError> {
    let result = AttendanceService::bulk_check_in(
        &state.db,
        body.service_id,
        body.member_ids,
        Some(user.0.id),
    )
    .await?;
    Ok(Json(result))
}

#[debug_handler]
async fn delete_attendance(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    AttendanceService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
