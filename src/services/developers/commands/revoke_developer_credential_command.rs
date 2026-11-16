use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::developer_application_repository::credentials,
};

pub async fn revoke_developer_credential(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
    credential_id: Uuid,
) -> Result<(), AppError> {
    credentials::revoke_developer_credential(pool, owner_account_id, application_id, credential_id)
        .await
}
