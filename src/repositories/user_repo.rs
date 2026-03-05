use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, role)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(
        pool: &PgPool,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                email = COALESCE($2, email),
                password_hash = COALESCE($3, password_hash),
                role = COALESCE($4, role),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
