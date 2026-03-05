use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::dto::household_dto::{
    CreateHouseholdRequest, HouseholdQuery, HouseholdResponse, LinkMemberRequest,
    UpdateHouseholdRequest,
};
use crate::dto::member_dto::MemberResponse;
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::extractors::AppJson;
use crate::middleware::auth::{RequireAdmin, RequireStaff};
use crate::services::household_service::HouseholdService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_households).post(create_household))
        .route(
            "/:id",
            get(get_household)
                .patch(update_household)
                .delete(delete_household),
        )
        .route("/:id/members", get(get_household_members))
        .route(
            "/:id/members/:member_id",
            put(link_member).delete(unlink_member),
        )
}

async fn list_households(
    State(state): State<AppState>,
    _user: RequireStaff,
    Query(query): Query<HouseholdQuery>,
) -> Result<Json<ListResponse<HouseholdResponse>>, AppError> {
    let households = HouseholdService::find_all(&state.db, query).await?;
    Ok(Json(households))
}

async fn create_household(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    AppJson(body): AppJson<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<HouseholdResponse>), AppError> {
    body.validate()?;
    let household = HouseholdService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(household)))
}

async fn get_household(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<HouseholdResponse>, AppError> {
    let household = HouseholdService::find_by_id(&state.db, id).await?;
    Ok(Json(household))
}

async fn update_household(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
    AppJson(body): AppJson<UpdateHouseholdRequest>,
) -> Result<Json<HouseholdResponse>, AppError> {
    body.validate()?;
    let household = HouseholdService::update(&state.db, id, body).await?;
    Ok(Json(household))
}

async fn delete_household(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    HouseholdService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_household_members(
    State(state): State<AppState>,
    _user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let members = HouseholdService::get_members(&state.db, id).await?;
    Ok(Json(members))
}

async fn link_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path((id, member_id)): Path<(Uuid, Uuid)>,
    AppJson(body): AppJson<LinkMemberRequest>,
) -> Result<Json<MemberResponse>, AppError> {
    body.validate()?;
    let member = HouseholdService::link_member(
        &state.db,
        id,
        member_id,
        body.household_role.as_deref(),
    )
    .await?;
    Ok(Json(member))
}

async fn unlink_member(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path((id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MemberResponse>, AppError> {
    let member = HouseholdService::unlink_member(&state.db, id, member_id).await?;
    Ok(Json(member))
}
