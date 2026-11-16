use sqlx::PgPool;

use crate::{
    enums::agent::AgentReviewStatus, errors::app_error::AppError,
    models::agent_identity::AgentReviewRecord, repositories::agent_administration_repository,
};

pub async fn list_agents_for_administrative_review(
    pool: &PgPool,
    status: Option<AgentReviewStatus>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<AgentReviewRecord>, i64), AppError> {
    let status = status.map(AgentReviewStatus::as_database_value);
    agent_administration_repository::list_agents_for_administrative_review(
        pool,
        status,
        i64::from(limit),
        i64::from(offset),
    )
    .await
}
