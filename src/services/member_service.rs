use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::member_dto::{CreateMemberRequest, MemberQuery, MemberResponse, UpdateMemberRequest};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::repositories::member_repo::MemberRepository;

pub struct MemberService;

impl MemberService {
    pub async fn create(
        pool: &PgPool,
        req: CreateMemberRequest,
    ) -> Result<MemberResponse, AppError> {
        let membership_status = req.membership_status.as_deref().unwrap_or("active");

        let member = MemberRepository::create(
            pool,
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

        Ok(member.into())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<MemberResponse, AppError> {
        let member = MemberRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member.into())
    }

    pub async fn find_all(
        pool: &PgPool,
        query: MemberQuery,
    ) -> Result<ListResponse<MemberResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let members = MemberRepository::find_all(
            pool,
            query.search.as_deref(),
            query.membership_status.as_deref(),
            query.household_id,
            limit,
            offset,
        )
        .await?;

        let total = MemberRepository::count(
            pool,
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
        pool: &PgPool,
        id: Uuid,
        req: UpdateMemberRequest,
    ) -> Result<MemberResponse, AppError> {
        let member = MemberRepository::update(
            pool,
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

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !MemberRepository::soft_delete(pool, id).await? {
            return Err(AppError::NotFound("Member not found".to_string()));
        }
        Ok(())
    }
}
