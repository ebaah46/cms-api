pub mod config;
pub mod dto;
pub mod errors;
pub mod extractors;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use config::Config;
use moka::future::Cache;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    repositories::{
        AttendanceRepository, GroupRepository, HouseholdRepository, MemberRepository,
        PostgresAttendanceRepository, PostgresGroupRepository, PostgresHouseholdRepository,
        PostgresMemberRepository, PostgresRefreshTokenRepository, PostgresServiceRepository,
        PostgresUserRepository, RefreshTokenRepository, ServiceRepository, UserRepository,
    },
    services::{
        attendance_service::CachedAttendanceService, auth_service::AuthService,
        group_service::GroupService, household_service::HouseholdService,
        import_service::ImportService, member_service::CachedMemberService,
        service_service::CachedServiceService, user_service::CachedUserService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub user_service: Arc<CachedUserService>,
    pub member_service: Arc<CachedMemberService>,
    pub attendance_service: Arc<CachedAttendanceService>,
    pub auth_service: Arc<AuthService>,
    pub group_service: Arc<GroupService>,
    pub household_service: Arc<HouseholdService>,
    pub import_service: Arc<ImportService>,
    pub service_service: Arc<CachedServiceService>,
}

pub struct AppStateBuilder {
    config: Arc<Config>,
    user_service: Option<Arc<CachedUserService>>,
    member_service: Option<Arc<CachedMemberService>>,
    attendance_service: Option<Arc<CachedAttendanceService>>,
    auth_service: Option<Arc<AuthService>>,
    group_service: Option<Arc<GroupService>>,
    household_service: Option<Arc<HouseholdService>>,
    import_service: Option<Arc<ImportService>>,
    service_service: Option<Arc<CachedServiceService>>,
}

impl AppStateBuilder {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            user_service: None,
            member_service: None,
            attendance_service: None,
            auth_service: None,
            group_service: None,
            household_service: None,
            import_service: None,
            service_service: None,
        }
    }

    pub fn user_service(mut self, user_service: Arc<CachedUserService>) -> Self {
        self.user_service = Some(user_service);
        self
    }

    pub fn member_service(mut self, member_service: Arc<CachedMemberService>) -> Self {
        self.member_service = Some(member_service);
        self
    }

    pub fn attendance_service(mut self, attendance_service: Arc<CachedAttendanceService>) -> Self {
        self.attendance_service = Some(attendance_service);
        self
    }
    pub fn auth_service(mut self, auth_service: Arc<AuthService>) -> Self {
        self.auth_service = Some(auth_service);
        self
    }
    pub fn group_service(mut self, group_service: Arc<GroupService>) -> Self {
        self.group_service = Some(group_service);
        self
    }
    pub fn household_service(mut self, household_service: Arc<HouseholdService>) -> Self {
        self.household_service = Some(household_service);
        self
    }

    pub fn import_service(mut self, import_service: Arc<ImportService>) -> Self {
        self.import_service = Some(import_service);
        self
    }

    pub fn service_service(mut self, service_service: Arc<CachedServiceService>) -> Self {
        self.service_service = Some(service_service);
        self
    }

    pub fn build(self) -> AppState {
        AppState {
            config: self.config,
            user_service: self.user_service.expect("User service not found"),
            member_service: self.member_service.expect("Member service not found"),
            attendance_service: self
                .attendance_service
                .expect("Attendance service not found"),
            auth_service: self.auth_service.expect("Auth service not found"),
            group_service: self.group_service.expect("Group service not found"),
            household_service: self.household_service.expect("Household service not found"),
            import_service: self.import_service.expect("Import service not found"),
            service_service: self.service_service.expect("Service service not found"),
        }
    }
}

pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn get_user_repo(&self) -> Arc<dyn UserRepository> {
        Arc::new(PostgresUserRepository::new(self.pool.clone()))
    }

    pub fn get_attendance_repo(&self) -> Arc<dyn AttendanceRepository> {
        Arc::new(PostgresAttendanceRepository::new(self.pool.clone()))
    }

    pub fn get_member_repo(&self) -> Arc<dyn MemberRepository> {
        Arc::new(PostgresMemberRepository::new(self.pool.clone()))
    }

    pub fn get_household_repo(&self) -> Arc<dyn HouseholdRepository> {
        Arc::new(PostgresHouseholdRepository::new(self.pool.clone()))
    }

    pub fn get_group_repo(&self) -> Arc<dyn GroupRepository> {
        Arc::new(PostgresGroupRepository::new(self.pool.clone()))
    }

    pub fn get_service_repo(&self) -> Arc<dyn ServiceRepository> {
        Arc::new(PostgresServiceRepository::new(self.pool.clone()))
    }

    pub fn get_refresh_token_repo(&self) -> Arc<dyn RefreshTokenRepository> {
        Arc::new(PostgresRefreshTokenRepository::new(self.pool.clone()))
    }
}

pub trait Cacheable: Send + Sync {
    fn cache_key(&self) -> String;

    fn cache_key_from_id<I>(id: I) -> String
    where
        I: Into<String>;
}

#[derive(Clone)]
pub struct CacheManager<T> {
    inner: Cache<String, T>,
}

impl<T: Cacheable + Clone + 'static> CacheManager<T> {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner: cache }
    }

    pub async fn get_entry<I>(&self, key: I) -> Option<T>
    where
        I: Into<String>,
    {
        self.inner.get(&key.into()).await
    }

    pub async fn set_entry_with_method<I, F, Fut, E>(&self, key: I, func: F) -> Result<T, E>
    where
        I: Into<String>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let entry = func().await?;
        self.inner.insert(key.into(), entry.clone()).await;
        Ok(entry)
    }

    pub async fn set_entry<I>(&self, key: I, value: T)
    where
        I: Into<String>,
    {
        self.inner.insert(key.into(), value).await
    }

    pub async fn invalidate_cache<I>(&self, key: I)
    where
        I: Into<String>,
    {
        self.inner.invalidate(&key.into()).await
    }
}
