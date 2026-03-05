use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::group::Group;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGroupRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 50, message = "Group type must be between 1 and 50 characters"))]
    pub group_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGroupRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50, message = "Group type must be between 1 and 50 characters"))]
    pub group_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub group_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Group> for GroupResponse {
    fn from(g: Group) -> Self {
        GroupResponse {
            id: g.id,
            name: g.name,
            group_type: g.group_type,
            description: g.description,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GroupQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub group_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddMemberToGroupRequest {
    #[validate(length(max = 50, message = "Role must be at most 50 characters"))]
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupMemberResponse {
    pub member_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}
