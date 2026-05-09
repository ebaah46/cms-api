use std::sync::Arc;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::household_dto::{
    CreateHouseholdRequest, HouseholdQuery, HouseholdResponse, UpdateHouseholdRequest,
};
use crate::dto::member_dto::MemberResponse;
use crate::errors::AppError;
use crate::repositories::HouseholdRepository;
use crate::repositories::MemberRepository;

pub struct HouseholdService {
    household_repo: Arc<dyn HouseholdRepository>,
    member_repo: Arc<dyn MemberRepository>,
}

impl HouseholdService {
    pub fn new(
        household_repo: Arc<dyn HouseholdRepository>,
        member_repo: Arc<dyn MemberRepository>,
    ) -> Self {
        Self {
            household_repo,
            member_repo,
        }
    }
    pub async fn create(&self, req: CreateHouseholdRequest) -> Result<HouseholdResponse, AppError> {
        let household = self
            .household_repo
            .create(&req.name, req.address.as_deref(), req.phone.as_deref())
            .await?;

        Ok(household.into())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<HouseholdResponse, AppError> {
        let household = self
            .household_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        Ok(household.into())
    }

    pub async fn find_all(
        &self,
        query: HouseholdQuery,
    ) -> Result<ListResponse<HouseholdResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let households = self
            .household_repo
            .find_all(query.search.as_deref(), limit, offset)
            .await?;
        let total = self.household_repo.count(query.search.as_deref()).await?;

        Ok(ListResponse {
            data: households.into_iter().map(|h| h.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateHouseholdRequest,
    ) -> Result<HouseholdResponse, AppError> {
        let household = self
            .household_repo
            .update(
                id,
                req.name.as_deref(),
                req.address.as_deref(),
                req.phone.as_deref(),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        Ok(household.into())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.household_repo.delete(id).await? {
            return Err(AppError::NotFound("Household not found".to_string()));
        }
        Ok(())
    }

    pub async fn get_members(&self, household_id: Uuid) -> Result<Vec<MemberResponse>, AppError> {
        // Verify household exists
        self.household_repo
            .find_by_id(household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        let members = self.member_repo.find_by_household(household_id).await?;
        Ok(members.into_iter().map(|m| m.into()).collect())
    }

    pub async fn link_member(
        &self,
        household_id: Uuid,
        member_id: Uuid,
        household_role: Option<&str>,
    ) -> Result<MemberResponse, AppError> {
        // Verify household exists
        self.household_repo
            .find_by_id(household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        let member = self
            .member_repo
            .update_household(member_id, Some(household_id), household_role)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn unlink_member(
        &self,
        household_id: Uuid,
        member_id: Uuid,
    ) -> Result<MemberResponse, AppError> {
        // Verify household exists
        self.household_repo
            .find_by_id(household_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Household not found".to_string()))?;

        // Verify member belongs to this household
        let member = self
            .member_repo
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        if member.household_id != Some(household_id) {
            return Err(AppError::BadRequest(
                "Member does not belong to this household".to_string(),
            ));
        }

        let updated_member = self
            .member_repo
            .update_household(member_id, None, None)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(updated_member.into())
    }
}
