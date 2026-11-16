use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_APPROVED_AUDIT_ACTION, DEVELOPER_APPLICATION_AUDIT_TARGET_TYPE,
        DEVELOPER_APPLICATION_REJECTED_AUDIT_ACTION,
    },
    enums::developer::{DeveloperApplicationReviewDecision, DeveloperError},
    errors::app_error::AppError,
};

pub async fn review_developer_application(
    pool: &PgPool,
    application_id: Uuid,
    administrator_id: Uuid,
    decision: DeveloperApplicationReviewDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let application_status: Option<String> = sqlx::query_scalar(
        "SELECT application_status FROM developer_applications
         WHERE id = $1 FOR UPDATE",
    )
    .bind(application_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some(application_status) = application_status else {
        return Err(DeveloperError::ApplicationNotFound.into());
    };
    if application_status != "submitted" {
        return Err(DeveloperError::ApplicationNotAwaitingReview.into());
    }

    let status = decision.as_database_value();
    sqlx::query(
        "UPDATE developer_applications
         SET application_status = $2, reviewed_by_administrator_id = $3,
             reviewed_at = now(), review_reason = $4
         WHERE id = $1",
    )
    .bind(application_id)
    .bind(status)
    .bind(administrator_id)
    .bind(reason)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE developer_application_requested_scopes
         SET scope_status = $2 WHERE application_id = $1 AND scope_status = 'requested'",
    )
    .bind(application_id)
    .bind(status)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    let audit_action = match decision {
        DeveloperApplicationReviewDecision::Approve => DEVELOPER_APPLICATION_APPROVED_AUDIT_ACTION,
        DeveloperApplicationReviewDecision::Reject => DEVELOPER_APPLICATION_REJECTED_AUDIT_ACTION,
    };
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
             result, reason, correlation_id)
         VALUES ($1, $2, $3, $4, $5, 'success', $6, $7)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(audit_action)
    .bind(DEVELOPER_APPLICATION_AUDIT_TARGET_TYPE)
    .bind(application_id)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
