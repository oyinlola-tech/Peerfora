use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::moderation_repository};

pub async fn assign_moderation_report(
    pool: &PgPool,
    report_id: Uuid,
    assigned_administrator_id: Uuid,
    actor_administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    moderation_repository::assign_report(
        pool,
        report_id,
        assigned_administrator_id,
        actor_administrator_id,
        correlation_id,
    )
    .await
}
