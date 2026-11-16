use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::AGENT_OPERATOR_CLAIM_TOKEN_LENGTH, enums::agent_error::AgentError,
    errors::app_error::AppError, repositories::agent_identity_repository,
    repositories::authentication_rate_limit_repository::hash_authentication_identifier,
    utils::one_time_token::hash_one_time_token,
};

pub async fn complete_agent_operator_claim(
    pool: &PgPool,
    operator_account_id: Uuid,
    operator_email: &str,
    token: &str,
    authentication_rate_limit_hash_key: &str,
) -> Result<Uuid, AppError> {
    if token.len() != AGENT_OPERATOR_CLAIM_TOKEN_LENGTH
        || !token.bytes().all(|character| character.is_ascii_hexdigit())
    {
        return Err(AgentError::AgentClaimInvalid.into());
    }
    let operator_email_hash =
        hash_authentication_identifier(operator_email, authentication_rate_limit_hash_key)
            .map_err(|_| AppError::Internal)?;
    agent_identity_repository::complete_agent_operator_claim(
        pool,
        operator_account_id,
        &operator_email_hash,
        &hash_one_time_token(token),
    )
    .await
}
