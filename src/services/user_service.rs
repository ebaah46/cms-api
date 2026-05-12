use std::sync::Arc;
use uuid::Uuid;

use crate::dto::auth_dto::{SetupRequest, SetupResponse};
use crate::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::dto::{ListResponse, PaginationParams};
use crate::errors::AppError;
use crate::models::user::UserRole;
use crate::repositories::UserRepository;
use crate::services::auth_service::AuthService;
use crate::{CacheManager, Cacheable};

struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    // Creates new instance of user service
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    /// Creates the initial admin user. Only works when no users exist in the database.
    async fn setup(&self, req: SetupRequest) -> Result<SetupResponse, AppError> {
        // Check if any users already exist
        let user_count = self.repo.count().await?;
        if user_count > 0 {
            return Err(AppError::Forbidden);
        }

        // Create the first admin user
        let password_hash = AuthService::hash_password(&req.password)?;
        let user = self
            .repo
            .create(&req.email, &password_hash, UserRole::SuperAdmin.as_str())
            .await?;

        Ok(SetupResponse {
            message: "Initial admin user created successfully".to_string(),
            user_id: user.id.to_string(),
            email: user.email,
        })
    }

    async fn create(&self, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        // Check if email already exists
        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let password_hash = AuthService::hash_password(&req.password)?;
        let role = req.role.unwrap_or(UserRole::Staff);

        let user = self
            .repo
            .create(&req.email, &password_hash, role.as_str())
            .await?;

        Ok(user.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    async fn find_all(
        &self,
        pagination: PaginationParams,
    ) -> Result<ListResponse<UserResponse>, AppError> {
        let users = self
            .repo
            .find_all(pagination.limit(), pagination.offset())
            .await?;
        let total = self.repo.count().await?;

        Ok(ListResponse {
            data: users.into_iter().map(|u| u.into()).collect(),
            total,
            page: pagination.page(),
            limit: pagination.limit(),
        })
    }

    async fn update(&self, id: Uuid, req: UpdateUserRequest) -> Result<UserResponse, AppError> {
        // Check if user exists
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check if new email already exists (if changing email)
        if let Some(ref email) = req.email {
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("Email already exists".to_string()));
                }
            }
        }

        let password_hash = match &req.password {
            Some(password) => Some(AuthService::hash_password(password)?),
            None => None,
        };

        let user = self
            .repo
            .update(
                id,
                req.email.as_deref(),
                password_hash.as_deref(),
                req.role.as_ref().map(|r| r.as_str()),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.repo.delete(id).await? {
            return Err(AppError::NotFound("User not found".to_string()));
        }
        Ok(())
    }
}

pub struct CachedUserService {
    inner: UserService,
    cache: CacheManager<UserResponse>,
}

impl CachedUserService {
    const MAX_CACHE_CAPACITY: u64 = 1000; // number of users cache can store
    const CACHE_TIME_TO_LIVE_SECS: u64 = 600; // number of secs entry can live until eviction based on strategy

    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self {
            inner: UserService::new(repo),
            cache: CacheManager::new(Self::MAX_CACHE_CAPACITY, Self::CACHE_TIME_TO_LIVE_SECS),
        }
    }

    pub async fn setup(&self, req: SetupRequest) -> Result<SetupResponse, AppError> {
        self.inner.setup(req).await
    }

    pub async fn create(&self, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        self.inner.create(req).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let key = UserResponse::cache_key_from_id(id);
        if let Some(val) = self.cache.get_entry(&key).await {
            return Ok(val);
        }
        let closure = move || self.inner.find_by_id(id);
        self.cache.set_entry(key, closure).await
    }

    pub async fn find_all(
        &self,
        pagination: PaginationParams,
    ) -> Result<ListResponse<UserResponse>, AppError> {
        self.inner.find_all(pagination).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateUserRequest) -> Result<UserResponse, AppError> {
        self.inner.update(id, req).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.inner.delete(id).await
    }
}
