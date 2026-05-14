use std::sync::Arc;
use uuid::Uuid;

use crate::dto::ListResponse;
use crate::dto::service_dto::{
    CreateServiceRequest, ServiceQuery, ServiceResponse, UpdateServiceRequest,
};
use crate::errors::AppError;
use crate::repositories::ServiceRepository;
use crate::{CacheManager, Cacheable};

pub struct ServiceService {
    repo: Arc<dyn ServiceRepository>,
}

impl ServiceService {
    fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    async fn create(&self, req: CreateServiceRequest) -> Result<ServiceResponse, AppError> {
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

    async fn find_by_id(&self, id: Uuid) -> Result<ServiceResponse, AppError> {
        let service = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        Ok(service.into())
    }

    async fn find_all(
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

    async fn update(
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

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if !self.repo.delete(id).await? {
            return Err(AppError::NotFound("Service not found".to_string()));
        }
        Ok(())
    }
}

pub struct CachedServiceService {
    inner: ServiceService,
    cache: CacheManager<ServiceResponse>,
}

impl CachedServiceService {
    const MAX_CACHE_CAPACITY: u64 = 10; // number of services cache can store
    const CACHE_TIME_TO_LIVE_SECS: u64 = 600; // number of secs entry can live until eviction based on strategy

    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self {
            inner: ServiceService::new(repo),
            cache: CacheManager::new(Self::MAX_CACHE_CAPACITY, Self::CACHE_TIME_TO_LIVE_SECS),
        }
    }

    pub async fn create(&self, req: CreateServiceRequest) -> Result<ServiceResponse, AppError> {
        let service = self.inner.create(req).await?;
        let key = ServiceResponse::cache_key_from_id(service.id);
        self.cache.set_entry(key, service.clone()).await;
        Ok(service)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<ServiceResponse, AppError> {
        let key = ServiceResponse::cache_key_from_id(id);
        if let Some(service) = self.cache.get_entry(&key).await {
            return Ok(service);
        }
        let closure = move || self.inner.find_by_id(id);
        self.cache.set_entry_with_method(key, closure).await
    }

    pub async fn find_all(
        &self,
        query: ServiceQuery,
    ) -> Result<ListResponse<ServiceResponse>, AppError> {
        self.inner.find_all(query).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateServiceRequest,
    ) -> Result<ServiceResponse, AppError> {
        let key = ServiceResponse::cache_key_from_id(id);
        self.cache.invalidate_cache(&key).await;
        let closure = move || self.inner.update(id, req);
        self.cache.set_entry_with_method(key, closure).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let key = ServiceResponse::cache_key_from_id(id);
        self.cache.invalidate_cache(&key).await;
        self.inner.delete(id).await
    }
}
