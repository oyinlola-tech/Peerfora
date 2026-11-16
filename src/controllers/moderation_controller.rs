use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    dtos::moderation::{
        AdministrativeModerationReportItem, AdministrativeModerationReportQuery,
        AdministrativeModerationReportsResponse, AssignModerationReportRequest,
        CreateModerationReportRequest, ModerationReportSubmissionResponse,
        ResolveModerationReportRequest,
    },
    enums::moderation::ModerationReportStatus,
    errors::app_error::AppError,
    models::{
        admin_identity::AdministratorSessionIdentity, app_state::AppState,
        moderation::ModerationReportTarget,
    },
    services::moderation::{
        commands::{
            assign_moderation_report_command, create_moderation_report_command,
            resolve_moderation_report_command,
        },
        queries::list_moderation_reports_query,
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn report_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::Account,
    )
    .await
}

pub async fn report_community(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::Community,
    )
    .await
}

pub async fn report_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::Post,
    )
    .await
}

pub async fn report_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::Comment,
    )
    .await
}

pub async fn report_research_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::ResearchProject,
    )
    .await
}

pub async fn report_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
    Json(request): Json<CreateModerationReportRequest>,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    submit_report(
        &state,
        &headers,
        target_id,
        request,
        ModerationReportTarget::ResearchFinding,
    )
    .await
}

async fn submit_report(
    state: &AppState,
    headers: &HeaderMap,
    target_id: Uuid,
    request: CreateModerationReportRequest,
    build_target: fn(Uuid) -> ModerationReportTarget,
) -> Result<(StatusCode, Json<ModerationReportSubmissionResponse>), AppError> {
    let account_id =
        crate::controllers::research_auth::require_authenticated_account_id(state, headers).await?;
    let (id, created_at) = create_moderation_report_command::create_moderation_report(
        &state.database,
        account_id,
        build_target(target_id),
        request.category,
        &request.description,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(ModerationReportSubmissionResponse {
            id,
            status: ModerationReportStatus::Open.as_database_value(),
            created_at,
        }),
    ))
}

pub async fn list_administrative_reports(
    State(state): State<AppState>,
    Query(query): Query<AdministrativeModerationReportQuery>,
) -> Result<Json<AdministrativeModerationReportsResponse>, AppError> {
    let (records, limit, offset, total_count) =
        list_moderation_reports_query::list_moderation_reports(
            &state.database,
            query.status,
            query.limit,
            query.offset,
        )
        .await?;
    Ok(Json(AdministrativeModerationReportsResponse {
        items: records
            .into_iter()
            .map(AdministrativeModerationReportItem::from)
            .collect(),
        total_count,
        limit,
        offset,
    }))
}

pub async fn resolve_administrative_report(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(report_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResolveModerationReportRequest>,
) -> Result<StatusCode, AppError> {
    resolve_moderation_report_command::resolve_moderation_report(
        &state.database,
        report_id,
        identity.administrator_id,
        request.decision,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_administrative_report(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(report_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AssignModerationReportRequest>,
) -> Result<StatusCode, AppError> {
    assign_moderation_report_command::assign_moderation_report(
        &state.database,
        report_id,
        request.administrator_id,
        identity.administrator_id,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
