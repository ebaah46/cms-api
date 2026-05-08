use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::member_dto::{
    CreateMemberRequest, MemberDetailResponse, MemberQuery, MemberResponse,
    UpdateMemberDetailRequest, UpdateMemberRequest,
};
use crate::errors::AppError;
use crate::repositories::MemberRepository;
use crate::repositories::member_repo::{
    CreateMemberParams, PostgresMemberRepository, UpdateMemberParams,
};

pub struct MemberService {
    repo: Arc<dyn MemberRepository>,
}

impl MemberService {
    pub async fn create(&self, req: CreateMemberRequest) -> Result<MemberResponse, AppError> {
        let membership_status = req.membership_status.as_deref().unwrap_or("active");

        let member = self
            .repo
            .create(
                &req.first_name,
                &req.last_name,
                req.email.as_deref(),
                req.phone.as_deref(),
                req.date_of_birth,
                req.gender.as_deref(),
                req.address.as_deref(),
                membership_status,
                req.membership_date,
                req.household_id,
                req.household_role.as_deref(),
            )
            .await?;

        // create associated member detail
        let mut member_detail = CreateMemberParams::default();
        member_detail.member_id = member.id;
        let _ = self.repo.create_detail(member_detail).await?;
        Ok(member.into())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<MemberResponse, AppError> {
        let member = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn find_detail_by_id(&self, id: Uuid) -> Result<MemberDetailResponse, AppError> {
        let member = self
            .repo
            .find_detail_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member Detail not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn find_all(
        &self,
        query: MemberQuery,
    ) -> Result<ListResponse<MemberResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let members = self
            .repo
            .find_all(
                query.search.as_deref(),
                query.membership_status.as_deref(),
                query.household_id,
                limit,
                offset,
            )
            .await?;

        let total = self
            .repo
            .count(
                query.search.as_deref(),
                query.membership_status.as_deref(),
                query.household_id,
            )
            .await?;

        Ok(ListResponse {
            data: members.into_iter().map(|m| m.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateMemberRequest,
    ) -> Result<MemberResponse, AppError> {
        let member = self
            .repo
            .update(
                id,
                req.first_name.as_deref(),
                req.last_name.as_deref(),
                req.email.as_deref(),
                req.phone.as_deref(),
                req.date_of_birth,
                req.gender.as_deref(),
                req.address.as_deref(),
                req.membership_status.as_deref(),
                req.membership_date,
                req.household_id,
                req.household_role.as_deref(),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn update_detail(
        &self,
        id: Uuid,
        req: UpdateMemberDetailRequest,
    ) -> Result<MemberDetailResponse, AppError> {
        let mut update_member_req = UpdateMemberParams::default();
        update_member_req.communicant = req.communicant;
        update_member_req.place_of_birth = req.place_of_birth;
        update_member_req.region_of_birth = req.region_of_birth;
        update_member_req.profession = req.profession;
        update_member_req.occupation = req.occupation;
        update_member_req.education_level = req.education_level;
        update_member_req.marital_status = req.marital_status;
        update_member_req.spouse_name = req.spouse_name;
        update_member_req.spouse_date_of_birth = req.spouse_date_of_birth;
        update_member_req.hometown = req.hometown;
        update_member_req.place_of_marriage = req.place_of_marriage;
        update_member_req.marriage_officiating_minister = req.marriage_officiating_minister;
        update_member_req.photo_url = req.photo_url;
        update_member_req.house_number = req.house_number;
        update_member_req.house_location = req.house_location;
        update_member_req.gps_address = req.gps_address;
        update_member_req.church = req.church;
        update_member_req.date_of_baptism = req.date_of_baptism;
        update_member_req.place_of_baptism = req.place_of_baptism;
        update_member_req.baptism_officiating_minister = req.baptism_officiating_minister;
        update_member_req.date_of_confirmation = req.date_of_confirmation;
        update_member_req.date_of_confirmation = req.date_of_confirmation;
        update_member_req.confirmation_officiating_minister = req.confirmation_officiating_minister;
        update_member_req.confirmation_text = req.confirmation_text;
        let member = self
            .repo
            .update_detail(id, update_member_req)
            .await?
            .ok_or_else(|| AppError::NotFound("Member Detail not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.repo.soft_delete(id).await? {
            return Err(AppError::NotFound("Member not found".to_string()));
        }
        Ok(())
    }
}
