use crate::{
    controllers::research_auth,
    dtos::research_project::{
        CreateResearchProjectRequest, InviteResearchProjectMemberRequest,
        ListResearchProjectsQuery, ResearchInvitationResponse, ResearchProjectResponse,
        UpdateResearchProjectRequest,
    },
    enums::research::ResearchProjectError,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::research::{
        commands::{
            accept_research_project_invitation_command, archive_research_project_command,
            create_research_project_command, invite_research_project_member_command,
            leave_research_project_command, reject_research_project_invitation_command,
            remove_research_project_member_command, update_research_project_command,
        },
        queries::{
            find_research_project_query, list_my_research_invitations_query,
            list_research_projects_query,
        },
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

pub async fn create_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateResearchProjectRequest>,
) -> Result<(StatusCode, Json<ResearchProjectResponse>), AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let project = create_research_project_command::create_research_project(
        &state.database,
        account_id,
        create_details(request),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(project.into())))
}

pub async fn list_research_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListResearchProjectsQuery>,
) -> Result<Json<Vec<ResearchProjectResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let projects = list_research_projects_query::list_research_projects_for_account(
        &state.database,
        account_id,
        query.limit,
    )
    .await?
    .into_iter()
    .map(ResearchProjectResponse::from)
    .collect();
    Ok(Json(projects))
}

pub async fn find_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ResearchProjectResponse>, AppError> {
    let account_id = research_auth::find_optional_account_id(&state, &headers).await?;
    let project = find_research_project_query::find_research_project_for_account(
        &state.database,
        project_id,
        account_id,
    )
    .await?
    .ok_or(ResearchProjectError::ProjectNotFound)?;
    Ok(Json(project.into()))
}

pub async fn update_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<UpdateResearchProjectRequest>,
) -> Result<Json<ResearchProjectResponse>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let project = update_research_project_command::update_research_project(
        &state.database,
        project_id,
        account_id,
        update_details(request),
    )
    .await?;
    Ok(Json(project.into()))
}

pub async fn archive_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    archive_research_project_command::archive_research_project(
        &state.database,
        project_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invite_research_project_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<InviteResearchProjectMemberRequest>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    invite_research_project_member_command::invite_research_project_member(
        &state.database,
        project_id,
        account_id,
        &request.username,
        request.role,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn list_my_research_invitations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ResearchInvitationResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let invitations = list_my_research_invitations_query::list_my_research_project_invitations(
        &state.database,
        account_id,
    )
    .await?
    .into_iter()
    .map(ResearchInvitationResponse::from)
    .collect();
    Ok(Json(invitations))
}

pub async fn accept_research_project_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    accept_research_project_invitation_command::accept_research_project_invitation(
        &state.database,
        project_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reject_research_project_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    reject_research_project_invitation_command::reject_research_project_invitation(
        &state.database,
        project_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    leave_research_project_command::leave_research_project(&state.database, project_id, account_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_research_project_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, username)): Path<(Uuid, String)>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    remove_research_project_member_command::remove_research_project_member(
        &state.database,
        project_id,
        account_id,
        &username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn create_details(
    request: CreateResearchProjectRequest,
) -> create_research_project_command::CreateResearchProjectDetails {
    create_research_project_command::CreateResearchProjectDetails {
        title: request.title,
        abstract_text: request.abstract_text,
        description: request.description,
        objectives: request.objectives,
        research_questions: request.research_questions,
        subject_ids: request.subject_ids,
        visibility: request.visibility,
    }
}

fn update_details(
    request: UpdateResearchProjectRequest,
) -> update_research_project_command::UpdateResearchProjectDetails {
    update_research_project_command::UpdateResearchProjectDetails {
        title: request.title,
        abstract_text: request.abstract_text,
        description: request.description,
        objectives: request.objectives,
        research_questions: request.research_questions,
        subject_ids: request.subject_ids,
    }
}
