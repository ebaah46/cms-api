use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::ListResponse;
use crate::dto::attendance_dto::{AttendanceQuery, AttendanceWithMemberResponse};
use crate::dto::service_dto::{
    CreateServiceRequest, ServiceQuery, ServiceResponse, UpdateServiceRequest,
};
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_services).post(create_service))
        .route(
            "/:id",
            get(get_service)
                .patch(update_service)
                .delete(delete_service),
        )
        .route("/:id/attendance", get(get_service_attendance))
}

async fn list_services(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<ServiceQuery>,
) -> Result<Json<ListResponse<ServiceResponse>>, AppError> {
    let services = state.service_service.find_all(query).await?;
    Ok(Json(services))
}

async fn create_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateServiceRequest>,
) -> Result<(StatusCode, Json<ServiceResponse>), AppError> {
    body.validate()?;
    let service = state.service_service.create(body).await?;
    Ok((StatusCode::CREATED, Json(service)))
}

async fn get_service(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceResponse>, AppError> {
    let service = state.service_service.find_by_id(id).await?;
    Ok(Json(service))
}

async fn update_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateServiceRequest>,
) -> Result<Json<ServiceResponse>, AppError> {
    body.validate()?;
    let service = state.service_service.update(id, body).await?;
    Ok(Json(service))
}

async fn delete_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.service_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_service_attendance(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<ListResponse<AttendanceWithMemberResponse>>, AppError> {
    let attendance = state
        .attendance_service
        .get_service_attendance(id, query.page, query.limit)
        .await?;
    Ok(Json(attendance))
}
