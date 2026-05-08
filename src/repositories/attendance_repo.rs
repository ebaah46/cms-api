use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::models::attendance::{Attendance, RefreshToken};

#[derive(Debug, FromRow)]
pub struct AttendanceWithMemberRow {
    pub id: Uuid,
    pub member_id: Uuid,
    pub member_name: String,
    pub service_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
    pub checked_in_by: Option<Uuid>,
}

#[async_trait]
pub trait AttendanceRepository: Send + Sync {
    async fn check_in(
        &self,
        member_id: Uuid,
        service_id: Uuid,
        checked_in_by: Option<Uuid>,
    ) -> Result<Attendance, sqlx::Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attendance>, sqlx::Error>;

    async fn find_by_member_and_service(
        &self,
        member_id: Uuid,
        service_id: Uuid,
    ) -> Result<Option<Attendance>, sqlx::Error>;

    async fn get_service_attendance(
        &self,
        service_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AttendanceWithMemberRow>, sqlx::Error>;

    async fn count_service_attendance(&self, service_id: Uuid) -> Result<i64, sqlx::Error>;

    async fn get_member_attendance(
        &self,
        member_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Attendance>, sqlx::Error>;

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}

pub struct PostgresAttendanceRepository {
    pool: PgPool,
}

#[async_trait]
impl AttendanceRepository for PostgresAttendanceRepository {
    async fn check_in(
        &self,
        member_id: Uuid,
        service_id: Uuid,
        checked_in_by: Option<Uuid>,
    ) -> Result<Attendance, sqlx::Error> {
        sqlx::query_as::<_, Attendance>(
            r#"
            INSERT INTO attendance (member_id, service_id, checked_in_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(member_id)
        .bind(service_id)
        .bind(checked_in_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attendance>, sqlx::Error> {
        sqlx::query_as::<_, Attendance>("SELECT * FROM attendance WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_member_and_service(
        &self,
        member_id: Uuid,
        service_id: Uuid,
    ) -> Result<Option<Attendance>, sqlx::Error> {
        sqlx::query_as::<_, Attendance>(
            "SELECT * FROM attendance WHERE member_id = $1 AND service_id = $2",
        )
        .bind(member_id)
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_service_attendance(
        &self,
        service_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AttendanceWithMemberRow>, sqlx::Error> {
        sqlx::query_as::<_, AttendanceWithMemberRow>(
            r#"
            SELECT a.id, a.member_id, CONCAT(m.first_name, ' ', m.last_name) as member_name,
                   a.service_id, a.checked_in_at, a.checked_in_by
            FROM attendance a
            JOIN members m ON m.id = a.member_id
            WHERE a.service_id = $1 AND m.deleted_at IS NULL
            ORDER BY a.checked_in_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(service_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_service_attendance(&self, service_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM attendance WHERE service_id = $1")
                .bind(service_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(result.0)
    }

    async fn get_member_attendance(
        &self,
        member_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Attendance>, sqlx::Error> {
        sqlx::query_as::<_, Attendance>(
            r#"
            SELECT * FROM attendance
            WHERE member_id = $1
            ORDER BY checked_in_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(member_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM attendance WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

pub struct RefreshTokenRepository;

impl RefreshTokenRepository {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT * FROM refresh_tokens WHERE token_hash = $1 AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    pub async fn revoke(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
