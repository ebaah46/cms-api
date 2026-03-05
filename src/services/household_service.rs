use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::household_dto::{
    CreateHouseholdRequest, HouseholdQuery, HouseholdResponse, UpdateHouseholdRequest,
};
use crate::dto::member_dto::MemberResponse;
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::repositories::household_repo::HouseholdRepository;
use crate::repositories::member_repo::MemberRepository;

pub struct HouseholdService;

impl HouseholdService {
    pub async fn create(
        pool: &PgPool,
        req: CreateHouseholdRequest,
    ) -> Result<HouseholdResponse, AppError> {
        let household = HouseholdRepository::create(
            pool,
            &req.name,
            req.address.as_deref(),
            req.phone.as_deref(),
        )
        .await?;

        Ok(household.into())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<HouseholdResponse, AppError> {
        let household = HouseholdRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        Ok(household.into())
    }

    pub async fn find_all(
        pool: &PgPool,
        query: HouseholdQuery,
    ) -> Result<ListResponse<HouseholdResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let households =
            HouseholdRepository::find_all(pool, query.search.as_deref(), limit, offset).await?;
        let total = HouseholdRepository::count(pool, query.search.as_deref()).await?;

        Ok(ListResponse {
            data: households.into_iter().map(|h| h.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateHouseholdRequest,
    ) -> Result<HouseholdResponse, AppError> {
        let household = HouseholdRepository::update(
            pool,
            id,
            req.name.as_deref(),
            req.address.as_deref(),
            req.phone.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        Ok(household.into())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !HouseholdRepository::delete(pool, id).await? {
            return Err(AppError::NotFound("Household not found".to_string()));
        }
        Ok(())
    }

    pub async fn get_members(
        pool: &PgPool,
        household_id: Uuid,
    ) -> Result<Vec<MemberResponse>, AppError> {
        // Verify household exists
        HouseholdRepository::find_by_id(pool, household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        let members = MemberRepository::find_by_household(pool, household_id).await?;
        Ok(members.into_iter().map(|m| m.into()).collect())
    }

    pub async fn link_member(
        pool: &PgPool,
        household_id: Uuid,
        member_id: Uuid,
        household_role: Option<&str>,
    ) -> Result<MemberResponse, AppError> {
        // Verify household exists
        HouseholdRepository::find_by_id(pool, household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        let member =
            MemberRepository::update_household(pool, member_id, Some(household_id), household_role)
                .await?
                .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn unlink_member(
        pool: &PgPool,
        household_id: Uuid,
        member_id: Uuid,
    ) -> Result<MemberResponse, AppError> {
        // Verify household exists
        HouseholdRepository::find_by_id(pool, household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        // Verify member belongs to this household
        let member = MemberRepository::find_by_id(pool, member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        if member.household_id != Some(household_id) {
            return Err(AppError::BadRequest(
                "Member does not belong to this household".to_string(),
            ));
        }

        let updated_member = MemberRepository::update_household(pool, member_id, None, None)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(updated_member.into())
    }
}
