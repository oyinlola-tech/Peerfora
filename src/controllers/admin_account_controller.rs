use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dtos::admin_auth::{
        AccountEnforcementRequest, AccountSessionsRevocationResponse, AdministrativeAccountItem,
        AdministrativeAccountQuery, AdministrativeAccountsResponse,
    },
    errors::app_error::AppError,
    models::admin_identity::AdministratorSessionIdentity,
    models::app_state::AppState,
    services::accounts::commands::{
        reinstate_account_command::reinstate_account, suspend_account_command::suspend_account,
    },
    services::administration::commands::revoke_account_sessions_command::revoke_account_sessions as revoke_account_sessions_for_account,
    services::administration::queries::search_accounts_for_administration_query::{
        AccountAdministrationFilters, find_account_for_administration,
        search_accounts_for_administration,
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn list_accounts(
    State(state): State<AppState>,
    Query(query): Query<AdministrativeAccountQuery>,
) -> Result<impl IntoResponse, AppError> {
    let filters = AccountAdministrationFilters {
        search_prefix: query.search,
        account_status: query.status,
        country_code: query.country_code,
        academic_level_id: query.academic_level_id,
        limit: query.limit,
        offset: query.offset,
    };
    let (records, limit, offset, total_count) =
        search_accounts_for_administration(&state.database, filters).await?;
    let items = records
        .into_iter()
        .map(AdministrativeAccountItem::from)
        .collect();
    Ok(Json(AdministrativeAccountsResponse {
        items,
        total_count,
        limit,
        offset,
    }))
}

pub async fn show_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let record = find_account_for_administration(&state.database, account_id).await?;
    Ok(Json(AdministrativeAccountItem::from(record)))
}

pub async fn suspend_account_access(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AccountEnforcementRequest>,
) -> Result<impl IntoResponse, AppError> {
    suspend_account(
        &state.database,
        account_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn reinstate_account_access(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AccountEnforcementRequest>,
) -> Result<impl IntoResponse, AppError> {
    reinstate_account(
        &state.database,
        account_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn revoke_account_sessions(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AccountEnforcementRequest>,
) -> Result<impl IntoResponse, AppError> {
    let revoked_session_count = revoke_account_sessions_for_account(
        &state.database,
        account_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(Json(AccountSessionsRevocationResponse {
        revoked_session_count,
    }))
}

impl From<crate::models::admin_identity::AdministrativeAccountRecord>
    for AdministrativeAccountItem
{
    fn from(record: crate::models::admin_identity::AdministrativeAccountRecord) -> Self {
        Self {
            id: record.id,
            email: record.email,
            username: record.username,
            status: record.account_status,
            email_verified: record.email_verified,
            country_code: record.country_code,
            academic_level_id: record.academic_level_id,
            created_at: record.created_at,
        }
    }
}
