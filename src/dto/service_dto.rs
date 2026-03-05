use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::service::Service;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServiceRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    pub service_date: NaiveDate,
    pub service_time: Option<NaiveTime>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateServiceRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    pub service_date: Option<NaiveDate>,
    pub service_time: Option<NaiveTime>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceResponse {
    pub id: Uuid,
    pub name: String,
    pub service_date: NaiveDate,
    pub service_time: Option<NaiveTime>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Service> for ServiceResponse {
    fn from(s: Service) -> Self {
        ServiceResponse {
            id: s.id,
            name: s.name,
            service_date: s.service_date,
            service_time: s.service_time,
            description: s.description,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServiceQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub search: Option<String>,
}
