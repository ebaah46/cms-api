use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::group_dto::{
    CreateGroupRequest, GroupMemberResponse, GroupQuery, GroupResponse, UpdateGroupRequest,
};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::repositories::group_repo::GroupRepository;
use crate::repositories::member_repo::MemberRepository;

pub struct GroupService;

impl GroupService {
    pub async fn create(pool: &PgPool, req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        let group = GroupRepository::create(
            pool,
            &req.name,
            &req.group_type,
            req.description.as_deref(),
        )
        .await?;

        Ok(group.into())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<GroupResponse, AppError> {
        let group = GroupRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        Ok(group.into())
    }

    pub async fn find_all(
        pool: &PgPool,
        query: GroupQuery,
    ) -> Result<ListResponse<GroupResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let groups = GroupRepository::find_all(
            pool,
            query.search.as_deref(),
            query.group_type.as_deref(),
            limit,
            offset,
        )
        .await?;

        let total =
            GroupRepository::count(pool, query.search.as_deref(), query.group_type.as_deref())
                .await?;

        Ok(ListResponse {
            data: groups.into_iter().map(|g| g.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError> {
        let group = GroupRepository::update(
            pool,
            id,
            req.name.as_deref(),
            req.group_type.as_deref(),
            req.description.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        Ok(group.into())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !GroupRepository::delete(pool, id).await? {
            return Err(AppError::NotFound("Group not found".to_string()));
        }
        Ok(())
    }

    pub async fn add_member(
        pool: &PgPool,
        group_id: Uuid,
        member_id: Uuid,
        role: Option<&str>,
    ) -> Result<(), AppError> {
        // Verify group exists
        GroupRepository::find_by_id(pool, group_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        // Verify member exists
        MemberRepository::find_by_id(pool, member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        GroupRepository::add_member(pool, group_id, member_id, role).await?;
        Ok(())
    }

    pub async fn remove_member(
        pool: &PgPool,
        group_id: Uuid,
        member_id: Uuid,
    ) -> Result<(), AppError> {
        if !GroupRepository::remove_member(pool, group_id, member_id).await? {
            return Err(AppError::NotFound(
                "Member not found in group".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn get_members(
        pool: &PgPool,
        group_id: Uuid,
    ) -> Result<Vec<GroupMemberResponse>, AppError> {
        // Verify group exists
        GroupRepository::find_by_id(pool, group_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        let members = GroupRepository::get_members(pool, group_id).await?;

        Ok(members
            .into_iter()
            .map(|m| GroupMemberResponse {
                member_id: m.member_id,
                first_name: m.first_name,
                last_name: m.last_name,
                email: m.email,
                role: m.role,
                joined_at: m.joined_at,
            })
            .collect())
    }
}
