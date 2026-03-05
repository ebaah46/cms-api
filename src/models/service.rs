use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub service_date: NaiveDate,
    pub service_time: Option<NaiveTime>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
