use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::attendance_dto::{
    AttendanceResponse, AttendanceWithMemberResponse, BulkCheckInResponse,
};
use crate::dto::ListResponse;
use crate::errors::AppError;
use crate::repositories::attendance_repo::AttendanceRepository;
use crate::repositories::member_repo::MemberRepository;
use crate::repositories::service_repo::ServiceRepository;

pub struct AttendanceService;

impl AttendanceService {
    pub async fn check_in(
        pool: &PgPool,
        member_id: Uuid,
        service_id: Uuid,
        checked_in_by: Option<Uuid>,
    ) -> Result<AttendanceResponse, AppError> {
        // Verify member exists
        MemberRepository::find_by_id(pool, member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        // Verify service exists
        ServiceRepository::find_by_id(pool, service_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        // Check if already checked in
        if AttendanceRepository::find_by_member_and_service(pool, member_id, service_id)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(
                "Member already checked in for this service".to_string(),
            ));
        }

        let attendance =
            AttendanceRepository::check_in(pool, member_id, service_id, checked_in_by).await?;

        Ok(attendance.into())
    }

    pub async fn bulk_check_in(
        pool: &PgPool,
        service_id: Uuid,
        member_ids: Vec<Uuid>,
        checked_in_by: Option<Uuid>,
    ) -> Result<BulkCheckInResponse, AppError> {
        // Verify service exists
        ServiceRepository::find_by_id(pool, service_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        let mut checked_in = 0;
        let mut already_checked_in = 0;
        let mut failed = 0;

        for member_id in member_ids {
            // Check if member exists
            if MemberRepository::find_by_id(pool, member_id)
                .await?
                .is_none()
            {
                failed += 1;
                continue;
            }

            // Check if already checked in
            if AttendanceRepository::find_by_member_and_service(pool, member_id, service_id)
                .await?
                .is_some()
            {
                already_checked_in += 1;
                continue;
            }

            // Check in
            match AttendanceRepository::check_in(pool, member_id, service_id, checked_in_by).await {
                Ok(_) => checked_in += 1,
                Err(_) => failed += 1,
            }
        }

        Ok(BulkCheckInResponse {
            checked_in,
            already_checked_in,
            failed,
        })
    }

    pub async fn get_service_attendance(
        pool: &PgPool,
        service_id: Uuid,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<ListResponse<AttendanceWithMemberResponse>, AppError> {
        // Verify service exists
        ServiceRepository::find_by_id(pool, service_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let attendance =
            AttendanceRepository::get_service_attendance(pool, service_id, limit, offset).await?;
        let total = AttendanceRepository::count_service_attendance(pool, service_id).await?;

        let data = attendance
            .into_iter()
            .map(|a| AttendanceWithMemberResponse {
                id: a.id,
                member_id: a.member_id,
                member_name: a.member_name,
                service_id: a.service_id,
                checked_in_at: a.checked_in_at,
                checked_in_by: a.checked_in_by,
            })
            .collect();

        Ok(ListResponse {
            data,
            total,
            page,
            limit,
        })
    }

    pub async fn get_member_attendance(
        pool: &PgPool,
        member_id: Uuid,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<AttendanceResponse>, AppError> {
        // Verify member exists
        MemberRepository::find_by_id(pool, member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let attendance =
            AttendanceRepository::get_member_attendance(pool, member_id, limit, offset).await?;

        Ok(attendance.into_iter().map(|a| a.into()).collect())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        if !AttendanceRepository::delete(pool, id).await? {
            return Err(AppError::NotFound("Attendance record not found".to_string()));
        }
        Ok(())
    }
}
