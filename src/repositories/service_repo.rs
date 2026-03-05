use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::service::Service;

pub struct ServiceRepository;

impl ServiceRepository {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        service_date: NaiveDate,
        service_time: Option<NaiveTime>,
        description: Option<&str>,
    ) -> Result<Service, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            INSERT INTO services (name, service_date, service_time, description)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(service_date)
        .bind(service_time)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>("SELECT * FROM services WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(
        pool: &PgPool,
        search: Option<&str>,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Service>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM services WHERE 1=1");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND service_date >= ${}", param_count));
        }

        if to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND service_date <= ${}", param_count));
        }

        query.push_str(&format!(
            " ORDER BY service_date DESC, service_time DESC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut q = sqlx::query_as::<_, Service>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(fd) = from_date {
            q = q.bind(fd);
        }
        if let Some(td) = to_date {
            q = q.bind(td);
        }

        q.bind(limit).bind(offset).fetch_all(pool).await
    }

    pub async fn count(
        pool: &PgPool,
        search: Option<&str>,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM services WHERE 1=1");
        let mut param_count = 0;

        if search.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND service_date >= ${}", param_count));
        }

        if to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND service_date <= ${}", param_count));
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&query);

        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(fd) = from_date {
            q = q.bind(fd);
        }
        if let Some(td) = to_date {
            q = q.bind(td);
        }

        let result = q.fetch_one(pool).await?;
        Ok(result.0)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<&str>,
        service_date: Option<NaiveDate>,
        service_time: Option<NaiveTime>,
        description: Option<&str>,
    ) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            UPDATE services
            SET
                name = COALESCE($2, name),
                service_date = COALESCE($3, service_date),
                service_time = COALESCE($4, service_time),
                description = COALESCE($5, description)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(service_date)
        .bind(service_time)
        .bind(description)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        // service cannot be deleted, will only be marked as removed
        let result = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
