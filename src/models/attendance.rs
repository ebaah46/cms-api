use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Attendance {
    pub id: Uuid,
    pub member_id: Uuid,
    pub service_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
    pub checked_in_by: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
