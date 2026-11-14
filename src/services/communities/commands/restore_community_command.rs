use uuid::Uuid;

use crate::{
    enums::{
        community::CommunityLifecycleStatus,
        community_enforcement_reason::CommunityEnforcementReason,
    },
    errors::app_error::AppError,
    repositories::community_administration_repository::change_community_lifecycle_status,
};

pub async fn restore_community(
    pool: &sqlx::PgPool,
    community_id: Uuid,
    administrator_id: Uuid,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = CommunityEnforcementReason::try_from(reason_code)?;
    change_community_lifecycle_status(
        pool,
        community_id,
        administrator_id,
        CommunityLifecycleStatus::Suspended,
        CommunityLifecycleStatus::Active,
        reason.as_database_value(),
        correlation_id,
    )
    .await
}
