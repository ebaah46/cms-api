pub mod attendance_dto;
pub mod auth_dto;
pub mod group_dto;
pub mod household_dto;
pub mod member_dto;
pub mod service_dto;
pub mod user_dto;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

impl PaginationParams {
    pub fn page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> i32 {
        (self.page() - 1) * self.limit()
    }
}
