use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::member::Member;

pub struct MemberRepository;

impl MemberRepository {
    pub async fn create(
        pool: &PgPool,
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
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
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(
        pool: &PgPool,
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

        q.bind(limit).bind(offset).fetch_all(pool).await
    }

    pub async fn count(
        pool: &PgPool,
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

        let result = q.fetch_one(pool).await?;
        Ok(result.0)
    }

    pub async fn find_by_household(
        pool: &PgPool,
        household_id: Uuid,
    ) -> Result<Vec<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            "SELECT * FROM members WHERE household_id = $1 AND deleted_at IS NULL ORDER BY last_name, first_name",
        )
        .bind(household_id)
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
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
                gender = COALESCE($7, gender),
                address = COALESCE($8, address),
                membership_status = COALESCE($9, membership_status),
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
        .fetch_optional(pool)
        .await
    }

    pub async fn update_household(
        pool: &PgPool,
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
        .fetch_optional(pool)
        .await
    }

    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE members SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
