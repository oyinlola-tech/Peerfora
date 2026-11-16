use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::agent_credential_repository};

pub async fn revoke_agent_credential(
    pool: &PgPool,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
    credential_id: Uuid,
) -> Result<(), AppError> {
    agent_credential_repository::revoke_agent_credential(
        pool,
        operator_account_id,
        agent_actor_id,
        credential_id,
    )
    .await
}
