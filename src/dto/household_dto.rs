use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::household::Household;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateHouseholdRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Phone must be at most 50 characters"))]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateHouseholdRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Phone must be at most 50 characters"))]
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HouseholdResponse {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Household> for HouseholdResponse {
    fn from(h: Household) -> Self {
        HouseholdResponse {
            id: h.id,
            name: h.name,
            address: h.address,
            phone: h.phone,
            created_at: h.created_at,
            updated_at: h.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HouseholdQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LinkMemberRequest {
    #[validate(length(min = 1, max = 50, message = "Household role must be at most 50 characters"))]
    pub household_role: Option<String>,
}
