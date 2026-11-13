use uuid::Uuid;

use crate::{
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
    models::admin_identity::AdministrativeAuditLogRecord, repositories::admin_auth_repository,
};

pub async fn list_administrative_audit_logs(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<(Vec<AdministrativeAuditLogRecord>, u32, u32), AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    if !(1..=100).contains(&limit) || offset > 100_000 {
        return Err(AdminAuthError::InvalidPagination.into());
    }
    let records = admin_auth_repository::list_administrative_audit_logs(
        pool,
        administrator_id,
        correlation_id,
        i64::from(limit),
        i64::from(offset),
    )
    .await?;
    Ok((records, limit, offset))
}
