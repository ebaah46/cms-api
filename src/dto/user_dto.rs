use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::user::{User, UserRole};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let role = user.role_enum();
        UserResponse {
            id: user.id,
            email: user.email,
            role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
