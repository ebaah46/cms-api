use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::models::user::UserRole;
use crate::services::auth_service::AuthService;

pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = AuthService::verify_access_token(token, &state.config)?;

        let user_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| AppError::Unauthorized)?;

        let role = UserRole::from_str(&claims.role).ok_or(AppError::Unauthorized)?;

        Ok(AuthUser {
            id: user_id,
            email: claims.email,
            role,
        })
    }
}

/// Requires any authenticated user (staff, admin, or superadmin)
/// Use for read-only operations
pub struct RequireStaff(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for RequireStaff {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        // All authenticated users (staff, admin, superadmin) can access
        Ok(RequireStaff(auth_user))
    }
}

/// Requires admin or superadmin role
/// Use for write operations (create, update, delete)
pub struct RequireAdmin(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        match auth_user.role {
            UserRole::SuperAdmin | UserRole::Admin => Ok(RequireAdmin(auth_user)),
            UserRole::Staff => Err(AppError::Forbidden),
        }
    }
}

pub struct RequireSuperAdmin(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for RequireSuperAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        match auth_user.role {
            UserRole::SuperAdmin => Ok(RequireSuperAdmin(auth_user)),
            _ => Err(AppError::Forbidden),
        }
    }
}
