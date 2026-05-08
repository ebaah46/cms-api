use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use tower::builder;
use uuid::Uuid;
use validator::ValidateRequired;

use crate::{
    models::member::{Member, MemberDetail},
    repositories::user_repo::PostgresUserRepository,
};

#[async_trait]
pub trait MemberRepository: Send + Sync {
    async fn create(
        &self,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        phone: Option<&str>,
        date_of_birth: Option<NaiveDate>,
        gender: Option<&str>,
        address: Option<&str>,
        membership_status: &str,
        membership_date: Option<NaiveDate>,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Member, sqlx::Error>;

    async fn create_detail(&self, params: CreateMemberParams) -> Result<MemberDetail, sqlx::Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Member>, sqlx::Error>;

    async fn find_detail_by_id(&self, id: Uuid) -> Result<Option<MemberDetail>, sqlx::Error>;

    async fn find_all(
        &self,
        search: Option<&str>,
        membership_status: Option<&str>,
        household_id: Option<Uuid>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Member>, sqlx::Error>;

    async fn count(
        &self,
        search: Option<&str>,
        membership_status: Option<&str>,
        household_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error>;

    async fn find_by_household(&self, household_id: Uuid) -> Result<Vec<Member>, sqlx::Error>;

    async fn update(
        &self,
        id: Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        date_of_birth: Option<NaiveDate>,
        gender: Option<&str>,
        address: Option<&str>,
        membership_status: Option<&str>,
        membership_date: Option<NaiveDate>,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Option<Member>, sqlx::Error>;

    async fn update_detail(
        &self,
        id: Uuid,
        params: UpdateMemberParams,
    ) -> Result<Option<MemberDetail>, sqlx::Error>;

    async fn update_household(
        &self,
        member_id: Uuid,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Option<Member>, sqlx::Error>;

    async fn soft_delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}

pub struct PostgresMemberRepository {
    pool: PgPool,
}

impl PostgresMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MemberRepository for PostgresMemberRepository {
    async fn create(
        &self,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        phone: Option<&str>,
        date_of_birth: Option<NaiveDate>,
        gender: Option<&str>,
        address: Option<&str>,
        membership_status: &str,
        membership_date: Option<NaiveDate>,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Member, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            r#"
            INSERT INTO members (
                first_name, last_name, email, phone, date_of_birth,
                gender, address, membership_status, membership_date,
                household_id, household_role
            )
            VALUES ($1, $2, $3, $4, $5, CAST($6 AS gender), $7, CAST($8 AS member_status), $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(phone)
        .bind(date_of_birth)
        .bind(if gender.is_none() {Some("unspecified")} else {gender})
        .bind(address)
        .bind(membership_status)
        .bind(membership_date)
        .bind(household_id)
        .bind(household_role)
        .fetch_one(&self.pool)
        .await
    }

    async fn create_detail(&self, params: CreateMemberParams) -> Result<MemberDetail, sqlx::Error> {
        sqlx::query_as::<_, MemberDetail>(
            r#"
            INSERT INTO member_details (
                member_id, communicant, place_of_birth, region_of_birth, education_level, profession,
                occupation, marital_status, spouse_name, spouse_date_of_birth, hometown, church, place_of_marriage,
                marriage_officiating_minister, date_of_baptism, place_of_baptism, baptism_officiating_minister,
                date_of_confirmation, place_of_confirmation, confirmation_officiating_minister, confirmation_text,
                photo_url, house_location, house_number, gps_address
            )
            VALUES ($1, $2, $3, $4, CAST($5 AS education_level), $6, $7, CAST($8 AS marital_status), $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            RETURNING *
            "#,
        ).bind(params.member_id).bind(params.communicant).bind(params.place_of_birth).bind(params.region_of_birth).bind(if params.education_level.is_some() {params.education_level} else { Some("none".into())}).bind(params.profession).bind(params.occupation)
        .bind(if params.marital_status.is_some() {params.marital_status} else {Some("single".into())})
        .bind(params.spouse_name).bind(params.spouse_date_of_birth).bind(params.hometown).bind(params.church).bind(params.place_of_marriage).bind(params.marriage_officiating_minister)
        .bind(params.date_of_baptism).bind(params.place_of_baptism).bind(params.baptism_officiating_minister).bind(params.date_of_confirmation).bind(params.place_of_confirmation)
        .bind(params.confirmation_officiating_minister).bind(params.confirmation_text).bind(params.photo_url).bind(params.house_location).bind(params.house_number)
        .bind(params.gps_address)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_detail_by_id(&self, id: Uuid) -> Result<Option<MemberDetail>, sqlx::Error> {
        sqlx::query_as::<_, MemberDetail>(
            "SELECT * FROM member_details WHERE member_id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(
        &self,
        search: Option<&str>,
        membership_status: Option<&str>,
        household_id: Option<Uuid>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Member>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM members WHERE deleted_at IS NULL");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(
                " AND (first_name ILIKE ${0} OR last_name ILIKE ${0} OR email ILIKE ${0})",
                param_count
            ));
        }

        if membership_status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND membership_status = ${}", param_count));
        }

        if household_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND household_id = ${}", param_count));
        }

        query.push_str(&format!(
            " ORDER BY last_name, first_name LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut q = sqlx::query_as::<_, Member>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(status) = membership_status {
            q = q.bind(status);
        }
        if let Some(hid) = household_id {
            q = q.bind(hid);
        }

        q.bind(limit).bind(offset).fetch_all(&self.pool).await
    }

    async fn count(
        &self,
        search: Option<&str>,
        membership_status: Option<&str>,
        household_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM members WHERE deleted_at IS NULL");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(
                " AND (first_name ILIKE ${0} OR last_name ILIKE ${0} OR email ILIKE ${0})",
                param_count
            ));
        }

        if membership_status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND membership_status = ${}", param_count));
        }

        if household_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND household_id = ${}", param_count));
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(status) = membership_status {
            q = q.bind(status);
        }
        if let Some(hid) = household_id {
            q = q.bind(hid);
        }

        let result = q.fetch_one(&self.pool).await?;
        Ok(result.0)
    }

    async fn find_by_household(&self, household_id: Uuid) -> Result<Vec<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            "SELECT * FROM members WHERE household_id = $1 AND deleted_at IS NULL ORDER BY last_name, first_name",
        )
        .bind(household_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        date_of_birth: Option<NaiveDate>,
        gender: Option<&str>,
        address: Option<&str>,
        membership_status: Option<&str>,
        membership_date: Option<NaiveDate>,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Option<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                email = COALESCE($4, email),
                phone = COALESCE($5, phone),
                date_of_birth = COALESCE($6, date_of_birth),
                gender = COALESCE(CAST($7 AS gender), gender),
                address = COALESCE($8, address),
                membership_status = COALESCE(CAST($9 AS member_status), membership_status),
                membership_date = COALESCE($10, membership_date),
                household_id = COALESCE($11, household_id),
                household_role = COALESCE($12, household_role),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(phone)
        .bind(date_of_birth)
        .bind(gender)
        .bind(address)
        .bind(membership_status)
        .bind(membership_date)
        .bind(household_id)
        .bind(household_role)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_detail(
        &self,
        id: Uuid,
        params: UpdateMemberParams,
    ) -> Result<Option<MemberDetail>, sqlx::Error> {
        sqlx::query_as::<_, MemberDetail>(
            r#"
            UPDATE member_details
            SET
                communicant = COALESCE($2, communicant),
                place_of_birth = COALESCE($3, place_of_birth),
                region_of_birth = COALESCE($4, region_of_birth),
                education_level = COALESCE($5, education_level),
                profession = COALESCE($6, profession),
                occupation = COALESCE($7, occupation),
                marital_status = COALESCE($8, marital_status),
                spouse_name = COALESCE($9, spouse_name),
                spouse_date_of_birth = COALESCE($10, spouse_date_of_birth),
                hometown = COALESCE($11, hometown),
                church = COALESCE($12, church),
                place_of_marriage = COALESCE($13, place_of_marriage),
                marriage_officiating_minister = COALESCE($14, marriage_officiating_minister),
                date_of_baptism = COALESCE($15, date_of_baptism),
                place_of_baptism = COALESCE($16, place_of_baptism),
                baptism_officiating_minister = COALESCE($17, baptism_officiating_minister),
                date_of_confirmation = COALESCE($18, date_of_confirmation),
                place_of_confirmation = COALESCE($19, place_of_confirmation),
                confirmation_officiating_minister = COALESCE($20, confirmation_officiating_minister),
                confirmation_text = COALESCE($21, confirmation_text),
                photo_url = COALESCE($22, photo_url),
                house_location = COALESCE($23, house_location),
                house_number = COALESCE($24, house_number),
                gps_address = COALESCE($25, gps_address),
                updated_at = NOW()
            WHERE member_id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(params.communicant)
        .bind(params.place_of_birth)
        .bind(params.region_of_birth)
        .bind(params.education_level)
        .bind(params.profession)
        .bind(params.occupation)
        .bind(params.marital_status)
        .bind(params.spouse_name)
        .bind(params.spouse_date_of_birth)
        .bind(params.hometown)
        .bind(params.church)
        .bind(params.place_of_marriage)
        .bind(params.marriage_officiating_minister)
        .bind(params.date_of_baptism)
        .bind(params.place_of_baptism)
        .bind(params.baptism_officiating_minister)
        .bind(params.date_of_confirmation)
        .bind(params.place_of_confirmation)
        .bind(params.confirmation_officiating_minister)
        .bind(params.confirmation_text)
        .bind(params.photo_url)
        .bind(params.house_location)
        .bind(params.house_number)
        .bind(params.gps_address)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_household(
        &self,
        member_id: Uuid,
        household_id: Option<Uuid>,
        household_role: Option<&str>,
    ) -> Result<Option<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET household_id = $2, household_role = $3, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(member_id)
        .bind(household_id)
        .bind(household_role)
        .fetch_optional(&self.pool)
        .await
    }

    async fn soft_delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            WITH updated_member AS (
                UPDATE members SET deleted_at = NOW(), updated_at = NOW()
                WHERE id = $1 AND deleted_at IS NULL
                RETURNING id
            )
            UPDATE member_details
            SET deleted_at = NOW(), updated_at = NOW()
            FROM updated_member
            WHERE member_details.member_id = updated_member.id;
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CreateMemberParams {
    pub member_id: Uuid,
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
    pub place_of_baptism: Option<String>,
    pub baptism_officiating_minister: Option<String>,
    pub date_of_confirmation: Option<NaiveDate>,
    pub place_of_confirmation: Option<String>,
    pub confirmation_officiating_minister: Option<String>,
    pub confirmation_text: Option<String>,
    pub photo_url: Option<String>,
    pub house_location: Option<String>,
    pub house_number: Option<String>,
    pub gps_address: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateMemberParams {
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
    pub place_of_baptism: Option<String>,
    pub baptism_officiating_minister: Option<String>,
    pub date_of_confirmation: Option<NaiveDate>,
    pub place_of_confirmation: Option<String>,
    pub confirmation_officiating_minister: Option<String>,
    pub confirmation_text: Option<String>,
    pub photo_url: Option<String>,
    pub house_location: Option<String>,
    pub house_number: Option<String>,
    pub gps_address: Option<String>,
}
