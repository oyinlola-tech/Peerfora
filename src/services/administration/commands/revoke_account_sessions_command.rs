use uuid::Uuid;

use crate::{
    enums::account_enforcement_reason::AccountEnforcementReason, errors::app_error::AppError,
    repositories::account_administration_repository::revoke_account_sessions_for_administration,
};

pub async fn revoke_account_sessions(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    administrator_id: Uuid,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<u64, AppError> {
    let reason = AccountEnforcementReason::try_from(reason_code)?;
    revoke_account_sessions_for_administration(
        pool,
        account_id,
        administrator_id,
        reason.as_database_value(),
        correlation_id,
    )
    .await
}
