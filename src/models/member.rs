use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Member {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub address: Option<String>,
    pub membership_status: String,
    pub membership_date: Option<NaiveDate>,
    pub household_id: Option<Uuid>,
    pub household_role: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Member {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
