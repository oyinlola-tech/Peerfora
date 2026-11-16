use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dtos::agent_credential::{
        AgentResearchTaskResponse, AgentResearchTaskSubmissionRequest,
        AgentResearchTaskSummaryResponse,
    },
    errors::app_error::AppError,
    models::{agent_credential::AgentCredentialIdentity, app_state::AppState},
    repositories::research_task_repository,
    services::research::{
        commands::{
            accept_agent_research_task_command, start_agent_research_task_command,
            submit_agent_research_task_result_command,
        },
        queries::list_agent_research_tasks_query,
    },
};

#[derive(Deserialize)]
pub struct AgentResearchTaskListQuery {
    pub limit: Option<i64>,
}

pub async fn list_assigned_research_tasks(
    State(state): State<AppState>,
    Extension(identity): Extension<AgentCredentialIdentity>,
    Query(query): Query<AgentResearchTaskListQuery>,
) -> Result<Json<Vec<AgentResearchTaskSummaryResponse>>, AppError> {
    let tasks = list_agent_research_tasks_query::list_research_tasks_assigned_to_agent(
        &state.database,
        identity.agent_actor_id,
        query.limit,
    )
    .await?
    .into_iter()
    .map(AgentResearchTaskSummaryResponse::from)
    .collect();
    Ok(Json(tasks))
}

pub async fn find_assigned_research_task(
    State(state): State<AppState>,
    Extension(identity): Extension<AgentCredentialIdentity>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<AgentResearchTaskResponse>, AppError> {
    let task = research_task_repository::find_assigned_research_task_for_agent(
        &state.database,
        task_id,
        identity.agent_actor_id,
    )
    .await?;
    Ok(Json(task.into()))
}

pub async fn accept_research_task(
    State(state): State<AppState>,
    Extension(identity): Extension<AgentCredentialIdentity>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    accept_agent_research_task_command::accept_research_task_assignment_for_agent(
        &state.database,
        task_id,
        identity.agent_actor_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_research_task(
    State(state): State<AppState>,
    Extension(identity): Extension<AgentCredentialIdentity>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    start_agent_research_task_command::start_research_task_for_agent(
        &state.database,
        task_id,
        identity.agent_actor_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn submit_research_task_result(
    State(state): State<AppState>,
    Extension(identity): Extension<AgentCredentialIdentity>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<AgentResearchTaskSubmissionRequest>,
) -> Result<StatusCode, AppError> {
    submit_agent_research_task_result_command::submit_research_task_result_for_agent(
        &state.database,
        task_id,
        identity.agent_actor_id,
        &request.result,
        request.sources,
    )
    .await?;
    Ok(StatusCode::CREATED)
}
