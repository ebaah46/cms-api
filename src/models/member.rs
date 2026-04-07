use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(sqlx::Type, Default, Debug, Clone)]
#[sqlx(type_name = "gender", rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    #[default]
    Unspecified,
}
impl From<String> for Gender {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "male" => Self::Male,
            "female" => Self::Female,
            _ => Self::Unspecified,
        }
    }
}
impl From<Gender> for String {
    fn from(gender: Gender) -> Self {
        match gender {
            Gender::Male => String::from("male"),
            Gender::Female => String::from("female"),
            Gender::Unspecified => String::from("unspecified"),
        }
    }
}

#[derive(sqlx::Type, Default, Debug, Clone)]
#[sqlx(type_name = "member_status", rename_all = "lowercase")]
pub enum MemberStatus {
    #[default]
    Active,
    InActive,
    Visitor,
    Transferred,
    Deceased,
}
impl From<String> for MemberStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "active" => Self::Active,
            "inactive" => Self::InActive,
            "visitor" => Self::Visitor,
            "transferred" => Self::Transferred,
            "deceased" => Self::Deceased,
            _ => Self::Active,
        }
    }
}
impl From<MemberStatus> for String {
    fn from(status: MemberStatus) -> Self {
        match status {
            MemberStatus::Active => String::from("active"),
            MemberStatus::InActive => String::from("inactive"),
            MemberStatus::Visitor => String::from("visitor"),
            MemberStatus::Transferred => String::from("transferred"),
            MemberStatus::Deceased => String::from("deceased"),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Member {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Gender,
    pub address: Option<String>,
    pub membership_status: MemberStatus,
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

pub struct MemberDetail {
    pub id: Uuid,
    pub member_id: Uuid,
    pub place_of_birth: Option<String>,
    pub region_of_birth: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub profession: Option<String>,
    pub occupation: Option<String>,
    pub marital_status: Option<MaritalStatus>,
    pub spouse_name: Option<String>,
    pub spouse_date_of_birth: Option<DateTime<Utc>>,
    pub hometown: Option<String>,
    pub church: Option<String>,
    pub place_of_marriage: Option<String>,
    pub marriage_officiating_minister: Option<String>,
    pub date_of_baptism: Option<DateTime<Utc>>,
    pub place_of_baptism: Option<String>,
    pub baptism_officiating_minister: Option<String>,
    // date_of_confirmation DATE,
    // place_of_confirmation VARCHAR(50),
    // confirmation_officiating_minister VARCHAR(100),
    // confirmation_text VARCHAR(50),
    // house_location VARCHAR(50),
    // house_number VARCHAR(50),
    // gps_address VARCHAR(50)
    // created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    // updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    // deleted_at TIMESTAMPTZ
}

impl MemberDetail {}

#[derive(sqlx::Type, Default, Debug, Clone)]
#[sqlx(type_name = "education_level", rename_all = "lowercase")]
pub enum EducationLevel {
    Primary,
    Jhs,
    Shs,
    Tetiary,
    #[default]
    None,
}
impl From<String> for EducationLevel {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "primary" => Self::Primary,
            "jhs" => Self::Jhs,
            "shs" => Self::Shs,
            "tetiary" => Self::Tetiary,
            _ => Self::None,
        }
    }
}
impl From<EducationLevel> for String {
    fn from(level: EducationLevel) -> Self {
        match level {
            EducationLevel::Primary => String::from("primary"),
            EducationLevel::Jhs => String::from("jhs"),
            EducationLevel::Shs => String::from("shs"),
            EducationLevel::Tetiary => String::from("tetiary"),
            _ => String::from("none"),
        }
    }
}

#[derive(sqlx::Type, Default, Debug, Clone)]
#[sqlx(type_name = "marital_status", rename_all = "lowercase")]
pub enum MaritalStatus {
    #[default]
    Single,
    Married,
    Divorced,
    Widowed,
    Separated,
}

impl From<String> for MaritalStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "single" => Self::Single,
            "married" => Self::Married,
            "divorced" => Self::Divorced,
            "widowed" => Self::Widowed,
            "separated" => Self::Separated,
            _ => Self::Single,
        }
    }
}
impl From<MaritalStatus> for String {
    fn from(status: MaritalStatus) -> Self {
        match status {
            MaritalStatus::Single => String::from("single"),
            MaritalStatus::Married => String::from("married"),
            MaritalStatus::Divorced => String::from("divorced"),
            MaritalStatus::Widowed => String::from("widowed"),
            MaritalStatus::Separated => String::from("separated"),
            _ => String::from("single"),
        }
    }
}
