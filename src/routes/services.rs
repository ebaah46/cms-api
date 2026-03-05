use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::dto::attendance_dto::{AttendanceQuery, AttendanceWithMemberResponse};
use crate::dto::service_dto::{
    CreateServiceRequest, ServiceQuery, ServiceResponse, UpdateServiceRequest,
};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};
use crate::services::attendance_service::AttendanceService;
use crate::services::service_service::ServiceService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_services).post(create_service))
        .route(
            "/:id",
            get(get_service).patch(update_service).delete(delete_service),
        )
        .route("/:id/attendance", get(get_service_attendance))
}

async fn list_services(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<ServiceQuery>,
) -> Result<Json<ListResponse<ServiceResponse>>, AppError> {
    let services = ServiceService::find_all(&state.db, query).await?;
    Ok(Json(services))
}

async fn create_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateServiceRequest>,
) -> Result<(StatusCode, Json<ServiceResponse>), AppError> {
    body.validate()?;
    let service = ServiceService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(service)))
}

async fn get_service(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceResponse>, AppError> {
    let service = ServiceService::find_by_id(&state.db, id).await?;
    Ok(Json(service))
}

async fn update_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateServiceRequest>,
) -> Result<Json<ServiceResponse>, AppError> {
    body.validate()?;
    let service = ServiceService::update(&state.db, id, body).await?;
    Ok(Json(service))
}

async fn delete_service(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ServiceService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_service_attendance(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<ListResponse<AttendanceWithMemberResponse>>, AppError> {
    let attendance =
        AttendanceService::get_service_attendance(&state.db, id, query.page, query.limit).await?;
    Ok(Json(attendance))
}
