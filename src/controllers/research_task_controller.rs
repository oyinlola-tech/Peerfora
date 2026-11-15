use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    controllers::research_auth,
    dtos::research_task::{
        AssignResearchTaskRequest, CreateResearchTaskRequest, ListResearchTasksQuery,
        ResearchTaskResponse, ReviewResearchTaskResultRequest, SubmitResearchTaskResultRequest,
        UpdateResearchTaskRequest,
    },
    errors::app_error::AppError,
    models::app_state::AppState,
    services::research::{
        commands::{
            accept_research_task_assignment_command, assign_research_task_command,
            cancel_research_task_command, create_research_task_command,
            queue_research_task_command, review_research_task_result_command,
            start_research_task_command, submit_research_task_result_command,
            update_research_task_command,
        },
        queries::{find_research_task_query, list_research_tasks_query},
        research_task_details::ResearchTaskDetails,
    },
};

pub async fn create_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CreateResearchTaskRequest>,
) -> Result<(StatusCode, Json<ResearchTaskResponse>), AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let task = create_research_task_command::create_research_task(
        &state.database,
        project_id,
        account_id,
        create_task_details(request),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(task.into())))
}

pub async fn list_research_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListResearchTasksQuery>,
) -> Result<Json<Vec<ResearchTaskResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let tasks = list_research_tasks_query::list_research_tasks_for_account(
        &state.database,
        project_id,
        account_id,
        query.limit,
    )
    .await?
    .into_iter()
    .map(ResearchTaskResponse::from)
    .collect();
    Ok(Json(tasks))
}

pub async fn find_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<Json<ResearchTaskResponse>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let task = find_research_task_query::find_research_task_for_account(
        &state.database,
        task_id,
        account_id,
    )
    .await?;
    Ok(Json(task.into()))
}

pub async fn update_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateResearchTaskRequest>,
) -> Result<Json<ResearchTaskResponse>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let task = update_research_task_command::update_research_task(
        &state.database,
        task_id,
        account_id,
        update_task_details(request),
    )
    .await?;
    Ok(Json(task.into()))
}

pub async fn queue_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    queue_research_task_command::queue_research_task(&state.database, task_id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(request): Json<AssignResearchTaskRequest>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    assign_research_task_command::assign_research_task(
        &state.database,
        task_id,
        account_id,
        &request.assignee_username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn accept_research_task_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    accept_research_task_assignment_command::accept_research_task_assignment(
        &state.database,
        task_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    start_research_task_command::start_research_task(&state.database, task_id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn submit_research_task_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(request): Json<SubmitResearchTaskResultRequest>,
) -> Result<Json<ResearchTaskResponse>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let task = submit_research_task_result_command::submit_research_task_result(
        &state.database,
        task_id,
        account_id,
        &request.result,
    )
    .await?;
    Ok(Json(task.into()))
}

pub async fn review_research_task_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(request): Json<ReviewResearchTaskResultRequest>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    review_research_task_result_command::review_research_task_result(
        &state.database,
        task_id,
        account_id,
        request.decision,
        request.notes.as_deref(),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_research_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    cancel_research_task_command::cancel_research_task(&state.database, task_id, account_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn create_task_details(request: CreateResearchTaskRequest) -> ResearchTaskDetails {
    ResearchTaskDetails {
        title: request.title,
        objective: request.objective,
        scope: request.scope,
        allowed_data_sources: request.allowed_data_sources,
        permitted_tools: request.permitted_tools,
        expected_deliverables: request.expected_deliverables,
        max_runtime_seconds: request.max_runtime_seconds,
        max_result_bytes: request.max_result_bytes,
    }
}

fn update_task_details(request: UpdateResearchTaskRequest) -> ResearchTaskDetails {
    ResearchTaskDetails {
        title: request.title,
        objective: request.objective,
        scope: request.scope,
        allowed_data_sources: request.allowed_data_sources,
        permitted_tools: request.permitted_tools,
        expected_deliverables: request.expected_deliverables,
        max_runtime_seconds: request.max_runtime_seconds,
        max_result_bytes: request.max_result_bytes,
    }
}
