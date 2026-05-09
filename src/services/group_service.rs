use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::group_dto::{
    CreateGroupRequest, GroupMemberResponse, GroupQuery, GroupResponse, UpdateGroupRequest,
};
use crate::errors::AppError;

use crate::repositories::{GroupRepository, MemberRepository};

pub struct GroupService {
    group_repo: Arc<dyn GroupRepository>,
    member_repo: Arc<dyn MemberRepository>,
}

impl GroupService {
    pub fn new(
        group_repo: Arc<dyn GroupRepository>,
        member_repo: Arc<dyn MemberRepository>,
    ) -> Self {
        Self {
            group_repo,
            member_repo,
        }
    }

    pub async fn create(&self, req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        let group = self
            .group_repo
            .create(&req.name, &req.group_type, req.description.as_deref())
            .await?;

        Ok(group.into())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<GroupResponse, AppError> {
        let group = self
            .group_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        Ok(group.into())
    }

    pub async fn find_all(
        &self,
        query: GroupQuery,
    ) -> Result<ListResponse<GroupResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let groups = self
            .group_repo
            .find_all(
                query.search.as_deref(),
                query.group_type.as_deref(),
                limit,
                offset,
            )
            .await?;

        let total = self
            .group_repo
            .count(query.search.as_deref(), query.group_type.as_deref())
            .await?;

        Ok(ListResponse {
            data: groups.into_iter().map(|g| g.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError> {
        let group = self
            .group_repo
            .update(
                id,
                req.name.as_deref(),
                req.group_type.as_deref(),
                req.description.as_deref(),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        Ok(group.into())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.group_repo.delete(id).await? {
            return Err(AppError::NotFound("Group not found".to_string()));
        }
        Ok(())
    }

    pub async fn add_member(
        &self,
        group_id: Uuid,
        member_id: Uuid,
        role: Option<&str>,
    ) -> Result<(), AppError> {
        // Verify group exists
        self.group_repo
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        // Verify member exists
        self.member_repo
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        self.group_repo
            .add_member(group_id, member_id, role)
            .await?;
        Ok(())
    }

    pub async fn remove_member(&self, group_id: Uuid, member_id: Uuid) -> Result<(), AppError> {
        if !self.group_repo.remove_member(group_id, member_id).await? {
            return Err(AppError::NotFound("Member not found in group".to_string()));
        }
        Ok(())
    }

    pub async fn get_members(&self, group_id: Uuid) -> Result<Vec<GroupMemberResponse>, AppError> {
        // Verify group exists
        self.group_repo
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        let members = self.group_repo.get_members(group_id).await?;

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

    pub async fn get_member_groups(&self, member_id: Uuid) -> Result<Vec<GroupResponse>, AppError> {
        let member_groups = self.group_repo.get_member_groups(member_id).await?;
        let gps: Vec<GroupResponse> = member_groups
            .into_iter()
            .map(|g| {
                let group: GroupResponse = g.into();
                group
            })
            .collect::<Vec<_>>();
        Ok(gps)
    }
}
