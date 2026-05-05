use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::member::{Member, MemberDetail};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMemberRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "First name must be between 1 and 100 characters"
    ))]
    pub first_name: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Last name must be between 1 and 100 characters"
    ))]
    pub last_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(max = 50, message = "Phone must be at most 50 characters"))]
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    #[validate(length(max = 20, message = "Gender must be at most 20 characters"))]
    pub gender: Option<String>,
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Membership status must be at most 50 characters"))]
    pub membership_status: Option<String>,
    pub membership_date: Option<NaiveDate>,
    pub household_id: Option<Uuid>,
    #[validate(length(max = 50, message = "Household role must be at most 50 characters"))]
    pub household_role: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMemberRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "First name must be between 1 and 100 characters"
    ))]
    pub first_name: Option<String>,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Last name must be between 1 and 100 characters"
    ))]
    pub last_name: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(max = 50, message = "Phone must be at most 50 characters"))]
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    #[validate(length(max = 20, message = "Gender must be at most 20 characters"))]
    pub gender: Option<String>,
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Membership status must be at most 50 characters"))]
    pub membership_status: Option<String>,
    pub membership_date: Option<NaiveDate>,
    pub household_id: Option<Uuid>,
    #[validate(length(max = 50, message = "Household role must be at most 50 characters"))]
    pub household_role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: String,
    pub address: Option<String>,
    pub membership_status: String,
    pub membership_date: Option<NaiveDate>,
    pub household_id: Option<Uuid>,
    pub household_role: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Member> for MemberResponse {
    fn from(m: Member) -> Self {
        MemberResponse {
            id: m.id,
            first_name: m.first_name,
            last_name: m.last_name,
            email: m.email,
            phone: m.phone,
            date_of_birth: m.date_of_birth,
            gender: m.gender.into(),
            address: m.address,
            membership_status: m.membership_status.into(),
            membership_date: m.membership_date,
            household_id: m.household_id,
            household_role: m.household_role,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MemberQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub membership_status: Option<String>,
    pub household_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub imported: i32,
    pub failed: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateMemberDetailRequest {
    pub communicant: bool,
    #[validate(length(max = 50, message = "Place of birth must be at most 50 characters"))]
    pub place_of_birth: Option<String>,
    #[validate(length(max = 50, message = "Region of birth must be at most 50 characters"))]
    pub region_of_birth: Option<String>,
    pub education_level: Option<String>,
    #[validate(length(max = 100, message = "Profession must be at most 100 characters"))]
    pub profession: Option<String>,
    #[validate(length(max = 100, message = "Occupation must be at most 100 characters"))]
    pub occupation: Option<String>,
    pub marital_status: Option<String>,
    #[validate(length(max = 100, message = "Spouse name must be at most 100 characters"))]
    pub spouse_name: Option<String>,
    pub spouse_date_of_birth: Option<NaiveDate>,
    pub hometown: Option<String>,
    pub church: Option<String>,
    pub place_of_marriage: Option<String>,
    pub marriage_officiating_minister: Option<String>,
    pub date_of_baptism: Option<NaiveDate>,
    pub baptism_officiating_minister: Option<String>,
    pub place_of_baptism: Option<String>,
    pub date_of_confirmation: Option<NaiveDate>,
    pub place_of_confirmation: Option<String>,
    pub confirmation_officiating_minister: Option<String>,
    #[validate(length(max = 50, message = "Confirmation text must be at most 50 characters"))]
    pub confirmation_text: Option<String>,
    pub photo_url: Option<String>,
    pub house_location: Option<String>,
    pub house_number: Option<String>,
    pub gps_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberDetailResponse {
    pub communicant: bool,
    pub place_of_birth: Option<String>,
    pub region_of_birth: Option<String>,
    pub education_level: Option<String>,
    pub profession: Option<String>,
    pub occupation: Option<String>,
    pub marital_status: Option<String>,
    pub spouse_name: Option<String>,
    pub spouse_date_of_birth: Option<NaiveDate>,
    pub hometown: Option<String>,
    pub church: Option<String>,
    pub place_of_marriage: Option<String>,
    pub marriage_officiating_minister: Option<String>,
    pub date_of_baptism: Option<NaiveDate>,
    pub baptism_officiating_minister: Option<String>,
    pub place_of_baptism: Option<String>,
    pub date_of_confirmation: Option<NaiveDate>,
    pub place_of_confirmation: Option<String>,
    pub confirmation_officiating_minister: Option<String>,
    pub confirmation_text: Option<String>,
    pub photo_url: Option<String>,
    pub house_location: Option<String>,
    pub house_number: Option<String>,
    pub gps_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MemberDetail> for MemberDetailResponse {
    fn from(value: MemberDetail) -> Self {
        MemberDetailResponse {
            communicant: value.communicant,
            place_of_birth: value.place_of_birth,
            region_of_birth: value.region_of_birth,
            education_level: value.education_level.map(|e| e.into()),
            profession: value.profession,
            occupation: value.occupation,
            marital_status: value.marital_status.map(|m| m.into()),
            spouse_name: value.spouse_name,
            spouse_date_of_birth: value.spouse_date_of_birth,
            hometown: value.hometown,
            church: value.church,
            place_of_marriage: value.place_of_marriage,
            marriage_officiating_minister: value.marriage_officiating_minister,
            date_of_baptism: value.date_of_baptism,
            baptism_officiating_minister: value.baptism_officiating_minister,
            place_of_baptism: value.place_of_baptism,
            date_of_confirmation: value.date_of_confirmation,
            place_of_confirmation: value.place_of_confirmation,
            confirmation_officiating_minister: value.confirmation_officiating_minister,
            confirmation_text: value.confirmation_text,
            photo_url: value.photo_url,
            house_location: value.house_location,
            house_number: value.house_number,
            gps_address: value.gps_address,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
