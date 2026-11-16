use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    constants::developers,
    controllers::research_auth::require_authenticated_account_id,
    dtos::developer::{
        ActivateDeveloperProfileRequest, AdministrativeDeveloperApplicationItem,
        AdministrativeDeveloperApplicationQuery, AdministrativeDeveloperApplicationsResponse,
        DeveloperApplicationResponse, DeveloperApplicationsResponse, DeveloperProfileResponse,
        RegisterDeveloperApplicationRequest, ReviewDeveloperApplicationRequest,
    },
    enums::developer::DeveloperProfileStatus,
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::developers::{
        commands::{
            activate_developer_profile_command, create_developer_application_command,
            review_developer_application_command, submit_developer_application_command,
        },
        queries::{
            list_developer_applications_for_review_query, list_developer_applications_query,
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn activate_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ActivateDeveloperProfileRequest>,
) -> Result<(StatusCode, Json<DeveloperProfileResponse>), AppError> {
    let account_id = require_authenticated_account_id(&state, &headers).await?;
    let profile = activate_developer_profile_command::activate_developer_profile(
        &state.database,
        account_id,
        &request.display_name,
        &request.description,
    )
    .await?;
    let profile_status = DeveloperProfileStatus::from_database_value(&profile.profile_status)
        .ok_or(AppError::Internal)?;
    Ok((
        StatusCode::CREATED,
        Json(DeveloperProfileResponse {
            account_id: profile.account_id,
            display_name: profile.display_name,
            description: profile.description,
            profile_status,
            created_at: profile.created_at,
        }),
    ))
}

pub async fn create_application(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterDeveloperApplicationRequest>,
) -> Result<(StatusCode, Json<DeveloperApplicationResponse>), AppError> {
    let account_id = require_authenticated_account_id(&state, &headers).await?;
    let record = create_developer_application_command::create_developer_application(
        &state.database,
        account_id,
        &request.name,
        &request.description,
        request.application_type,
        &request.requested_scopes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record.try_into()?)))
}

pub async fn list_my_applications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeveloperApplicationsResponse>, AppError> {
    let account_id = require_authenticated_account_id(&state, &headers).await?;
    let records =
        list_developer_applications_query::list_developer_applications(&state.database, account_id)
            .await?;
    let items = records
        .into_iter()
        .map(DeveloperApplicationResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(DeveloperApplicationsResponse { items }))
}

pub async fn submit_application(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(application_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = require_authenticated_account_id(&state, &headers).await?;
    submit_developer_application_command::submit_developer_application(
        &state.database,
        account_id,
        application_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_applications_for_review(
    State(state): State<AppState>,
    Query(query): Query<AdministrativeDeveloperApplicationQuery>,
) -> Result<Json<AdministrativeDeveloperApplicationsResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(developers::DEVELOPER_APPLICATION_DEFAULT_PAGE_SIZE)
        .clamp(1, developers::DEVELOPER_APPLICATION_MAX_PAGE_SIZE);
    let offset = query
        .offset
        .unwrap_or_default()
        .min(developers::DEVELOPER_APPLICATION_MAX_OFFSET);
    let (records, total_count) =
        list_developer_applications_for_review_query::list_developer_applications_for_review(
            &state.database,
            query.status,
            limit,
            offset,
        )
        .await?;
    let items = records
        .into_iter()
        .map(AdministrativeDeveloperApplicationItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(AdministrativeDeveloperApplicationsResponse {
        items,
        total_count,
        limit,
        offset,
    }))
}

pub async fn review_application(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    headers: HeaderMap,
    Path(application_id): Path<Uuid>,
    Json(request): Json<ReviewDeveloperApplicationRequest>,
) -> Result<StatusCode, AppError> {
    review_developer_application_command::review_developer_application(
        &state.database,
        application_id,
        identity.administrator_id,
        request.decision,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
