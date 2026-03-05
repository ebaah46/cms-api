use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Staff,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::SuperAdmin => "super_admin",
            UserRole::Admin => "admin",
            UserRole::Staff => "staff",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "super_admin" => Some(UserRole::SuperAdmin),
            "admin" => Some(UserRole::Admin),
            "staff" => Some(UserRole::Staff),
            _ => None,
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Staff
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn role_enum(&self) -> UserRole {
        UserRole::from_str(&self.role).unwrap_or_default()
    }
}
