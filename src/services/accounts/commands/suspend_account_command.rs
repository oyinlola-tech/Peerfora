use uuid::Uuid;

use crate::{
    enums::{
        account_enforcement_reason::AccountEnforcementReason,
        account_lifecycle::AccountLifecycleStatus,
    },
    errors::app_error::AppError,
    repositories::account_administration_repository::change_account_lifecycle_status,
};

pub async fn suspend_account(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    administrator_id: Uuid,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = AccountEnforcementReason::try_from(reason_code)?;
    change_account_lifecycle_status(
        pool,
        account_id,
        administrator_id,
        AccountLifecycleStatus::Active,
        AccountLifecycleStatus::Suspended,
        reason.as_database_value(),
        correlation_id,
    )
    .await
}
