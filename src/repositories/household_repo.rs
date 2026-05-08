use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::household::Household;

#[async_trait]
pub trait HouseholdRepository: Send + Sync {
    async fn create(
        &self,
        name: &str,
        address: Option<&str>,
        phone: Option<&str>,
    ) -> Result<Household, sqlx::Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Household>, sqlx::Error>;

    async fn find_all(
        &self,
        search: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Household>, sqlx::Error>;

    async fn count(&self, search: Option<&str>) -> Result<i64, sqlx::Error>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        address: Option<&str>,
        phone: Option<&str>,
    ) -> Result<Option<Household>, sqlx::Error>;

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}

pub struct PostgresHouseholdRepository {
    pool: PgPool,
}

impl PostgresHouseholdRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HouseholdRepository for PostgresHouseholdRepository {
    async fn create(
        &self,
        name: &str,
        address: Option<&str>,
        phone: Option<&str>,
    ) -> Result<Household, sqlx::Error> {
        sqlx::query_as::<_, Household>(
            r#"
            INSERT INTO households (name, address, phone)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(address)
        .bind(phone)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Household>, sqlx::Error> {
        sqlx::query_as::<_, Household>("SELECT * FROM households WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_all(
        &self,
        search: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Household>, sqlx::Error> {
        match search {
            Some(s) => {
                let pattern = format!("%{}%", s);
                sqlx::query_as::<_, Household>(
                    r#"
                    SELECT * FROM households
                    WHERE name ILIKE $1 OR address ILIKE $1
                    ORDER BY name
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(&pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Household>(
                    "SELECT * FROM households ORDER BY name LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    async fn count(&self, search: Option<&str>) -> Result<i64, sqlx::Error> {
        let result: (i64,) = match search {
            Some(s) => {
                let pattern = format!("%{}%", s);
                sqlx::query_as(
                    "SELECT COUNT(*) FROM households WHERE name ILIKE $1 OR address ILIKE $1",
                )
                .bind(&pattern)
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as("SELECT COUNT(*) FROM households")
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(result.0)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        address: Option<&str>,
        phone: Option<&str>,
    ) -> Result<Option<Household>, sqlx::Error> {
        sqlx::query_as::<_, Household>(
            r#"
            UPDATE households
            SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                phone = COALESCE($4, phone),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(address)
        .bind(phone)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM households WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
