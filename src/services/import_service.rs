use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::member_dto::ImportResult;
use crate::errors::AppError;
use crate::repositories::member_repo::MemberRepository;

#[derive(Debug, Deserialize)]
struct CsvMemberRow {
    first_name: String,
    last_name: String,
    email: Option<String>,
    phone: Option<String>,
    date_of_birth: Option<String>,
    gender: Option<String>,
    address: Option<String>,
    membership_status: Option<String>,
    membership_date: Option<String>,
}

pub struct ImportService;

impl ImportService {
    pub async fn import_members_from_csv(
        pool: &PgPool,
        csv_data: &[u8],
    ) -> Result<ImportResult, AppError> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data);

        let mut imported = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (index, result) in reader.deserialize().enumerate() {
            let row_num = index + 2; // +2 for 1-indexed and header row

            let row: CsvMemberRow = match result {
                Ok(r) => r,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Row {}: Failed to parse - {}", row_num, e));
                    continue;
                }
            };

            // Validate required fields
            if row.first_name.trim().is_empty() {
                failed += 1;
                errors.push(format!("Row {}: first_name is required", row_num));
                continue;
            }

            if row.last_name.trim().is_empty() {
                failed += 1;
                errors.push(format!("Row {}: last_name is required", row_num));
                continue;
            }

            // Parse dates
            let date_of_birth = Self::parse_date(&row.date_of_birth);
            let membership_date = Self::parse_date(&row.membership_date);

            let membership_status = row
                .membership_status
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("active");

            // Create member
            match MemberRepository::create(
                pool,
                row.first_name.trim(),
                row.last_name.trim(),
                row.email.as_deref().filter(|s| !s.is_empty()),
                row.phone.as_deref().filter(|s| !s.is_empty()),
                date_of_birth,
                row.gender.as_deref().filter(|s| !s.is_empty()),
                row.address.as_deref().filter(|s| !s.is_empty()),
                membership_status,
                membership_date,
                None, // household_id
                None, // household_role
            )
            .await
            {
                Ok(_) => imported += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "Row {}: Failed to create member {} {} - {}",
                        row_num, row.first_name, row.last_name, e
                    ));
                }
            }
        }

        Ok(ImportResult {
            imported,
            failed,
            errors,
        })
    }

    fn parse_date(date_str: &Option<String>) -> Option<NaiveDate> {
        date_str.as_ref().and_then(|s| {
            let s = s.trim();
            if s.is_empty() {
                return None;
            }

            // Try common date formats
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(s, "%m/%d/%Y"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%d/%m/%Y"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
                .ok()
        })
    }
}
