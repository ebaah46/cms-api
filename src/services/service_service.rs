use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::service_dto::{
    CreateServiceRequest, ServiceQuery, ServiceResponse, UpdateServiceRequest,
};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::repositories::service_repo::ServiceRepository;

pub struct ServiceService;

impl ServiceService {
    pub async fn create(
        pool: &PgPool,
        req: CreateServiceRequest,
    ) -> Result<ServiceResponse, AppError> {
        let service = ServiceRepository::create(
            pool,
            &req.name,
            req.service_date,
            req.service_time,
            req.description.as_deref(),
        )
        .await?;

        Ok(service.into())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<ServiceResponse, AppError> {
        let service = ServiceRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        Ok(service.into())
    }

    pub async fn find_all(
        pool: &PgPool,
        query: ServiceQuery,
    ) -> Result<ListResponse<ServiceResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let services = ServiceRepository::find_all(
            pool,
            query.search.as_deref(),
            query.from_date,
            query.to_date,
            limit,
            offset,
        )
        .await?;

        let total =
            ServiceRepository::count(pool, query.search.as_deref(), query.from_date, query.to_date)
                .await?;

        Ok(ListResponse {
            data: services.into_iter().map(|s| s.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateServiceRequest,
    ) -> Result<ServiceResponse, AppError> {
        let service = ServiceRepository::update(
            pool,
            id,
            req.name.as_deref(),
            req.service_date,
            req.service_time,
            req.description.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        Ok(service.into())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !ServiceRepository::delete(pool, id).await? {
            return Err(AppError::NotFound("Service not found".to_string()));
        }
        Ok(())
    }
}
