use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::auth_dto::{SetupRequest, SetupResponse};
use crate::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::dto::{ListResponse, PaginationParams};
use crate::errors::AppError;
use crate::models::user::UserRole;
use crate::repositories::user_repo::UserRepository;
use crate::services::auth_service::AuthService;

pub struct UserService;

impl UserService {
    /// Creates the initial admin user. Only works when no users exist in the database.
    pub async fn setup(pool: &PgPool, req: SetupRequest) -> Result<SetupResponse, AppError> {
        // Check if any users already exist
        let user_count = UserRepository::count(pool).await?;
        if user_count > 0 {
            return Err(AppError::Forbidden);
        }

        // Create the first admin user
        let password_hash = AuthService::hash_password(&req.password)?;
        let user =
            UserRepository::create(pool, &req.email, &password_hash, UserRole::SuperAdmin.as_str())
                .await?;

        Ok(SetupResponse {
            message: "Initial admin user created successfully".to_string(),
            user_id: user.id.to_string(),
            email: user.email,
        })
    }

    pub async fn create(pool: &PgPool, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        // Check if email already exists
        if UserRepository::find_by_email(pool, &req.email)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let password_hash = AuthService::hash_password(&req.password)?;
        let role = req.role.unwrap_or(UserRole::Staff);

        let user =
            UserRepository::create(pool, &req.email, &password_hash, role.as_str()).await?;

        Ok(user.into())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<UserResponse, AppError> {
        let user = UserRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    pub async fn find_all(
        pool: &PgPool,
        pagination: PaginationParams,
    ) -> Result<ListResponse<UserResponse>, AppError> {
        let users = UserRepository::find_all(pool, pagination.limit(), pagination.offset()).await?;
        let total = UserRepository::count(pool).await?;

        Ok(ListResponse {
            data: users.into_iter().map(|u| u.into()).collect(),
            total,
            page: pagination.page(),
            limit: pagination.limit(),
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        // Check if user exists
        UserRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check if new email already exists (if changing email)
        if let Some(ref email) = req.email {
            if let Some(existing) = UserRepository::find_by_email(pool, email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("Email already exists".to_string()));
                }
            }
        }

        let password_hash = match &req.password {
            Some(password) => Some(AuthService::hash_password(password)?),
            None => None,
        };

        let user = UserRepository::update(
            pool,
            id,
            req.email.as_deref(),
            password_hash.as_deref(),
            req.role.as_ref().map(|r| r.as_str()),
        )
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !UserRepository::delete(pool, id).await? {
            return Err(AppError::NotFound("User not found".to_string()));
        }
        Ok(())
    }
}
