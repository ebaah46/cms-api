use std::sync::Arc;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::service_dto::{
    CreateServiceRequest, ServiceQuery, ServiceResponse, UpdateServiceRequest,
};
use crate::errors::AppError;
use crate::repositories::ServiceRepository;

pub struct ServiceService {
    repo: Arc<dyn ServiceRepository>,
}

impl ServiceService {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, req: CreateServiceRequest) -> Result<ServiceResponse, AppError> {
        let service = self
            .repo
            .create(
                &req.name,
                req.service_date,
                req.service_time,
                req.description.as_deref(),
            )
            .await?;

        Ok(service.into())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<ServiceResponse, AppError> {
        let service = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        Ok(service.into())
    }

    pub async fn find_all(
        &self,
        query: ServiceQuery,
    ) -> Result<ListResponse<ServiceResponse>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let services = self
            .repo
            .find_all(
                query.search.as_deref(),
                query.from_date,
                query.to_date,
                limit,
                offset,
            )
            .await?;

        let total = self
            .repo
            .count(query.search.as_deref(), query.from_date, query.to_date)
            .await?;

        Ok(ListResponse {
            data: services.into_iter().map(|s| s.into()).collect(),
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateServiceRequest,
    ) -> Result<ServiceResponse, AppError> {
        let service = self
            .repo
            .update(
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

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.repo.delete(id).await? {
            return Err(AppError::NotFound("Service not found".to_string()));
        }
        Ok(())
    }
}
