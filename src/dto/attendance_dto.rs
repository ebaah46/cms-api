use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::attendance::Attendance;

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub member_id: Uuid,
    pub service_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct BulkCheckInRequest {
    pub service_id: Uuid,
    pub member_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub service_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
    pub checked_in_by: Option<Uuid>,
}

impl From<Attendance> for AttendanceResponse {
    fn from(a: Attendance) -> Self {
        AttendanceResponse {
            id: a.id,
            member_id: a.member_id,
            service_id: a.service_id,
            checked_in_at: a.checked_in_at,
            checked_in_by: a.checked_in_by,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AttendanceWithMemberResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub member_name: String,
    pub service_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
    pub checked_in_by: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BulkCheckInResponse {
    pub checked_in: i32,
    pub already_checked_in: i32,
    pub failed: i32,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}
