use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::group_dto::{
    CreateGroupRequest, GroupMemberResponse, GroupQuery, GroupResponse, UpdateGroupRequest,
};
use crate::errors::AppError;
use crate::{CacheManager, Cacheable};

use crate::repositories::{GroupRepository, MemberRepository};

struct GroupService {
    group_repo: Arc<dyn GroupRepository>,
    member_repo: Arc<dyn MemberRepository>,
}

impl GroupService {
    fn new(group_repo: Arc<dyn GroupRepository>, member_repo: Arc<dyn MemberRepository>) -> Self {
        Self {
            group_repo,
            member_repo,
        }
    }

    async fn create(&self, req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        let group = self
            .group_repo
            .create(&req.name, &req.group_type, req.description.as_deref())
            .await?;

        Ok(group.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<GroupResponse, AppError> {
        let group = self
            .group_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        Ok(group.into())
    }

    async fn find_all(&self, query: GroupQuery) -> Result<ListResponse<GroupResponse>, AppError> {
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

    async fn update(&self, id: Uuid, req: UpdateGroupRequest) -> Result<GroupResponse, AppError> {
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

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.group_repo.delete(id).await? {
            return Err(AppError::NotFound("Group not found".to_string()));
        }
        Ok(())
    }

    async fn add_member(
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

    async fn remove_member(&self, group_id: Uuid, member_id: Uuid) -> Result<(), AppError> {
        if !self.group_repo.remove_member(group_id, member_id).await? {
            return Err(AppError::NotFound("Member not found in group".to_string()));
        }
        Ok(())
    }

    async fn get_members(&self, group_id: Uuid) -> Result<Vec<GroupMemberResponse>, AppError> {
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

    async fn get_member_groups(&self, member_id: Uuid) -> Result<Vec<GroupResponse>, AppError> {
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

pub struct CachedGroupService {
    inner: GroupService,
    cache: CacheManager<GroupResponse>,
}

impl CachedGroupService {
    const MAX_CACHE_CAPACITY: u64 = 20; // number of groups cache can store
    const CACHE_TIME_TO_LIVE_SECS: u64 = 600; // number of secs entry can live until eviction based on strategy

    pub fn new(
        group_repo: Arc<dyn GroupRepository>,
        member_repo: Arc<dyn MemberRepository>,
    ) -> Self {
        Self {
            inner: GroupService::new(group_repo, member_repo),
            cache: CacheManager::new(Self::MAX_CACHE_CAPACITY, Self::CACHE_TIME_TO_LIVE_SECS),
        }
    }

    pub async fn create(&self, req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        let group = self.inner.create(req).await?;
        let key = GroupResponse::cache_key_from_id(group.id);
        self.cache.set_entry(key, group.clone()).await;
        Ok(group)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<GroupResponse, AppError> {
        let key = GroupResponse::cache_key_from_id(id);
        if let Some(group) = self.cache.get_entry(&key).await {
            return Ok(group);
        }
        let closure = move || self.inner.find_by_id(id);
        self.cache.set_entry_with_method(key, closure).await
    }

    pub async fn find_all(
        &self,
        query: GroupQuery,
    ) -> Result<ListResponse<GroupResponse>, AppError> {
        self.inner.find_all(query).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError> {
        let key = GroupResponse::cache_key_from_id(id);
        self.cache.invalidate_cache(&key).await;
        let closure = move || self.inner.update(id, req);
        self.cache.set_entry_with_method(key, closure).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let key = GroupResponse::cache_key_from_id(id);
        self.cache.invalidate_cache(&key).await;
        self.inner.delete(id).await
    }

    pub async fn add_member(
        &self,
        group_id: Uuid,
        member_id: Uuid,
        role: Option<&str>,
    ) -> Result<(), AppError> {
        self.inner.add_member(group_id, member_id, role).await
    }

    pub async fn remove_member(&self, group_id: Uuid, member_id: Uuid) -> Result<(), AppError> {
        self.inner.remove_member(group_id, member_id).await
    }

    pub async fn get_members(&self, group_id: Uuid) -> Result<Vec<GroupMemberResponse>, AppError> {
        self.inner.get_members(group_id).await
    }

    pub async fn get_member_groups(&self, member_id: Uuid) -> Result<Vec<GroupResponse>, AppError> {
        self.inner.get_member_groups(member_id).await
    }
}
