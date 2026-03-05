use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::models::group::{Group, MemberGroup};

#[derive(Debug, FromRow)]
pub struct GroupMemberRow {
    pub member_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}

pub struct GroupRepository;

impl GroupRepository {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        group_type: &str,
        description: Option<&str>,
    ) -> Result<Group, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO groups (name, group_type, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(group_type)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(
        pool: &PgPool,
        search: Option<&str>,
        group_type: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Group>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM groups WHERE 1=1");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if group_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND group_type = ${}", param_count));
        }

        query.push_str(&format!(
            " ORDER BY name LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut q = sqlx::query_as::<_, Group>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(gt) = group_type {
            q = q.bind(gt);
        }

        q.bind(limit).bind(offset).fetch_all(pool).await
    }

    pub async fn count(
        pool: &PgPool,
        search: Option<&str>,
        group_type: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM groups WHERE 1=1");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if group_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND group_type = ${}", param_count));
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(gt) = group_type {
            q = q.bind(gt);
        }

        let result = q.fetch_one(pool).await?;
        Ok(result.0)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<&str>,
        group_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            UPDATE groups
            SET
                name = COALESCE($2, name),
                group_type = COALESCE($3, group_type),
                description = COALESCE($4, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(group_type)
        .bind(description)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM groups WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn add_member(
        pool: &PgPool,
        group_id: Uuid,
        member_id: Uuid,
        role: Option<&str>,
    ) -> Result<MemberGroup, sqlx::Error> {
        sqlx::query_as::<_, MemberGroup>(
            r#"
            INSERT INTO member_groups (member_id, group_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (member_id, group_id) DO UPDATE SET role = $3
            RETURNING *
            "#,
        )
        .bind(member_id)
        .bind(group_id)
        .bind(role.unwrap_or("member"))
        .fetch_one(pool)
        .await
    }

    pub async fn remove_member(
        pool: &PgPool,
        group_id: Uuid,
        member_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM member_groups WHERE group_id = $1 AND member_id = $2")
                .bind(group_id)
                .bind(member_id)
                .execute(pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_members(
        pool: &PgPool,
        group_id: Uuid,
    ) -> Result<Vec<GroupMemberRow>, sqlx::Error> {
        sqlx::query_as::<_, GroupMemberRow>(
            r#"
            SELECT m.id as member_id, m.first_name, m.last_name, m.email, mg.role, mg.joined_at
            FROM member_groups mg
            JOIN members m ON m.id = mg.member_id
            WHERE mg.group_id = $1 AND m.deleted_at IS NULL
            ORDER BY m.last_name, m.first_name
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_member_groups(
        pool: &PgPool,
        member_id: Uuid,
    ) -> Result<Vec<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT g.*
            FROM groups g
            JOIN member_groups mg ON mg.group_id = g.id
            WHERE mg.member_id = $1
            ORDER BY g.name
            "#,
        )
        .bind(member_id)
        .fetch_all(pool)
        .await
    }
}
