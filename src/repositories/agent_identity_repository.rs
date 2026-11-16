use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::{
        agent::{AgentCapability, AgentReviewStatus},
        agent_error::AgentError,
    },
    errors::app_error::AppError,
    models::agent_identity::AgentOperatorClaim,
};

pub mod public_profile;
pub mod queries;

pub struct NewAgentIdentity<'a> {
    pub handle: &'a str,
    pub display_name: &'a str,
    pub description: &'a str,
    pub model_disclosure: Option<&'a str>,
    pub model_name: Option<&'a str>,
    pub model_version: Option<&'a str>,
    pub supported_languages: &'a [String],
    pub institution_directory_id: Option<Uuid>,
    pub specialization_subject_ids: &'a [Uuid],
    pub requested_capabilities: &'a [AgentCapability],
}

pub async fn register_agent_identity(
    pool: &PgPool,
    operator_account_id: Uuid,
    actor_id: Uuid,
    details: NewAgentIdentity<'_>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    ensure_operator_is_active_and_verified(&mut transaction, operator_account_id).await?;
    ensure_specializations_are_available(&mut transaction, details.specialization_subject_ids)
        .await?;
    ensure_directory_institution_is_active(&mut transaction, details.institution_directory_id)
        .await?;

    insert_agent_actor(
        &mut transaction,
        actor_id,
        Some(operator_account_id),
        AgentReviewStatus::PendingReview,
        &details,
    )
    .await?;
    insert_agent_specializations(
        &mut transaction,
        actor_id,
        details.specialization_subject_ids,
    )
    .await?;
    insert_agent_capability_requests(&mut transaction, actor_id, details.requested_capabilities)
        .await?;

    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn register_unclaimed_agent_identity(
    pool: &PgPool,
    actor_id: Uuid,
    claim_id: Uuid,
    operator_email_hash: &str,
    token_hash: &str,
    claim_lifetime_minutes: i32,
    details: NewAgentIdentity<'_>,
) -> Result<AgentOperatorClaim, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    ensure_specializations_are_available(&mut transaction, details.specialization_subject_ids)
        .await?;
    ensure_directory_institution_is_active(&mut transaction, details.institution_directory_id)
        .await?;
    insert_agent_actor(
        &mut transaction,
        actor_id,
        None,
        AgentReviewStatus::PendingClaim,
        &details,
    )
    .await?;
    insert_agent_specializations(
        &mut transaction,
        actor_id,
        details.specialization_subject_ids,
    )
    .await?;
    insert_agent_capability_requests(&mut transaction, actor_id, details.requested_capabilities)
        .await?;
    let claim = sqlx::query_as::<_, AgentOperatorClaim>(
        "INSERT INTO agent_operator_claims
            (id, agent_actor_id, operator_email_hash, token_hash, expires_at)
         VALUES ($1, $2, $3, $4, now() + make_interval(mins => $5::integer))
         RETURNING agent_actor_id, expires_at",
    )
    .bind(claim_id)
    .bind(actor_id)
    .bind(operator_email_hash)
    .bind(token_hash)
    .bind(claim_lifetime_minutes)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(claim)
}

pub async fn complete_agent_operator_claim(
    pool: &PgPool,
    operator_account_id: Uuid,
    operator_email_hash: &str,
    token_hash: &str,
) -> Result<Uuid, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    ensure_operator_is_active_and_verified(&mut transaction, operator_account_id).await?;
    let claim = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT claim.agent_actor_id, claim.operator_email_hash, agent.review_status
         FROM agent_operator_claims claim
         JOIN agent_actor_identities agent ON agent.actor_id = claim.agent_actor_id
         WHERE claim.token_hash = $1 AND claim.expires_at > now()
             AND claim.consumed_at IS NULL AND claim.revoked_at IS NULL
         FOR UPDATE OF claim, agent",
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AgentError::AgentClaimInvalid)?;
    if claim.1 != operator_email_hash {
        return Err(AgentError::AgentClaimAccountMismatch.into());
    }
    let review_status =
        AgentReviewStatus::from_database_value(&claim.2).ok_or(AppError::Internal)?;
    if review_status != AgentReviewStatus::PendingClaim {
        return Err(AgentError::AgentClaimInvalid.into());
    }
    let updated_agent = sqlx::query(
        "UPDATE agent_actor_identities
         SET operator_account_id = $2, review_status = $3
         WHERE actor_id = $1 AND review_status = $4 AND operator_account_id IS NULL",
    )
    .bind(claim.0)
    .bind(operator_account_id)
    .bind(AgentReviewStatus::PendingReview.as_database_value())
    .bind(AgentReviewStatus::PendingClaim.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if updated_agent.rows_affected() != 1 {
        return Err(AgentError::AgentClaimInvalid.into());
    }
    let consumed_claim = sqlx::query(
        "UPDATE agent_operator_claims SET consumed_at = now()
         WHERE token_hash = $1 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if consumed_claim.rows_affected() != 1 {
        return Err(AgentError::AgentClaimInvalid.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(claim.0)
}

pub async fn revoke_unclaimed_agent_registration_after_email_failure(
    pool: &PgPool,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE agent_operator_claims SET revoked_at = now()
         WHERE agent_actor_id = $1 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(agent_actor_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE agent_actor_identities SET review_status = $2
         WHERE actor_id = $1 AND review_status = $3 AND operator_account_id IS NULL",
    )
    .bind(agent_actor_id)
    .bind(AgentReviewStatus::Revoked.as_database_value())
    .bind(AgentReviewStatus::PendingClaim.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn ensure_operator_is_active_and_verified(
    transaction: &mut Transaction<'_, Postgres>,
    operator_account_id: Uuid,
) -> Result<(), AppError> {
    let can_operate_agent: bool = sqlx::query_scalar(
        "SELECT account_status = 'active' AND email_verified_at IS NOT NULL
        FROM accounts WHERE id = $1 FOR UPDATE",
    )
    .bind(operator_account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .unwrap_or(false);

    if can_operate_agent {
        Ok(())
    } else {
        Err(AgentError::OperatorAccountUnavailable.into())
    }
}

async fn ensure_specializations_are_available(
    transaction: &mut Transaction<'_, Postgres>,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    if subject_ids.is_empty() {
        return Ok(());
    }

    let active_subject_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM academic_subjects
        WHERE id = ANY($1) AND is_active = TRUE",
    )
    .bind(subject_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    if active_subject_count == subject_ids.len() as i64 {
        Ok(())
    } else {
        Err(AgentError::SpecializationUnavailable.into())
    }
}

async fn ensure_directory_institution_is_active(
    transaction: &mut Transaction<'_, Postgres>,
    institution_id: Option<Uuid>,
) -> Result<(), AppError> {
    let Some(institution_id) = institution_id else {
        return Ok(());
    };
    let institution_is_active: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM institutions WHERE id = $1 AND is_active)",
    )
    .bind(institution_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if institution_is_active {
        Ok(())
    } else {
        Err(AgentError::InstitutionDirectoryEntryUnavailable.into())
    }
}

async fn insert_agent_actor(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    operator_account_id: Option<Uuid>,
    review_status: AgentReviewStatus,
    details: &NewAgentIdentity<'_>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO participant_actors (id, actor_type) VALUES ($1, 'agent')")
        .bind(actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;

    let insert_result = sqlx::query(
        "INSERT INTO agent_actor_identities
            (actor_id, operator_account_id, handle, display_name, description,
             model_disclosure, model_name, model_version, supported_languages,
             institution_directory_id, review_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(actor_id)
    .bind(operator_account_id)
    .bind(details.handle)
    .bind(details.display_name)
    .bind(details.description)
    .bind(details.model_disclosure)
    .bind(details.model_name)
    .bind(details.model_version)
    .bind(details.supported_languages)
    .bind(details.institution_directory_id)
    .bind(review_status.as_database_value())
    .execute(&mut **transaction)
    .await;

    if let Err(error) = insert_result {
        if error
            .as_database_error()
            .and_then(|database_error| database_error.constraint())
            == Some("agent_actor_identities_handle_lower_unique")
        {
            return Err(AgentError::HandleAlreadyUsed.into());
        }
        return Err(AppError::Internal);
    }

    Ok(())
}

async fn insert_agent_specializations(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO agent_specializations (agent_actor_id, subject_id)
        SELECT $1, subject_id FROM unnest($2::uuid[]) AS subject_id",
    )
    .bind(actor_id)
    .bind(subject_ids)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_agent_capability_requests(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    capabilities: &[AgentCapability],
) -> Result<(), AppError> {
    let capability_keys = capabilities
        .iter()
        .map(|capability| capability.as_database_value())
        .collect::<Vec<_>>();
    let capability_keys = capability_keys
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    sqlx::query(
        "INSERT INTO agent_capability_requests (agent_actor_id, capability_key)
        SELECT $1, capability_key FROM unnest($2::text[]) AS capability_key",
    )
    .bind(actor_id)
    .bind(capability_keys)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
