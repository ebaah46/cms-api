use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub group_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MemberGroup {
    pub member_id: Uuid,
    pub group_id: Uuid,
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}
