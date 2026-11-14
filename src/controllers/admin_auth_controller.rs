use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, header},
    response::IntoResponse,
};

use crate::{
    constants::admin_auth::ADMIN_RECOVERY_CODES_NO_STORE_CACHE_CONTROL,
    dtos::admin_auth::{
        AdministrativeAuditLogItem, AdministrativeAuditLogQuery, AdministrativeAuditLogResponse,
        AdministratorChallengeRequest, AdministratorChallengeResponse, AdministratorLoginRequest,
        AdministratorMfaEnrollmentResponse, AdministratorMfaVerificationRequest,
        AdministratorOverviewResponse, AdministratorPasswordChangeRequest,
        AdministratorRecoveryCodeRegenerationRequest, AdministratorRecoveryCodesResponse,
        AdministratorSessionItem, AdministratorSessionResponse, AdministratorSessionsResponse,
    },
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::admin_auth::{
        commands::{
            begin_administrator_mfa_enrollment_command::begin_administrator_mfa_enrollment,
            change_administrator_password_command::change_administrator_password,
            complete_administrator_mfa_command::{
                confirm_administrator_mfa_enrollment, verify_administrator_mfa_login,
            },
            regenerate_administrator_recovery_codes_command::regenerate_administrator_recovery_codes,
            revoke_administrator_session_command::revoke_administrator_session,
            start_administrator_login_command::{
                challenge_lifetime_seconds, start_administrator_login,
            },
        },
        queries::{
            find_administrator_session_query::find_administrator_session,
            get_administrator_overview_query::get_administrator_overview,
            list_administrative_audit_logs_query::list_administrative_audit_logs,
            list_administrator_sessions_query::list_administrator_sessions,
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<AdministratorLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AdministratorLogin,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    let challenge =
        start_administrator_login(&state.database, &request.email, request.password).await?;
    Ok(Json(AdministratorChallengeResponse {
        challenge_token: challenge.challenge_token,
        next_step: challenge.next_step,
        expires_in_seconds: challenge_lifetime_seconds(),
    }))
}

pub async fn begin_mfa_enrollment(
    State(state): State<AppState>,
    Json(request): Json<AdministratorChallengeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let provisioning_uri = begin_administrator_mfa_enrollment(
        &state.database,
        &request.challenge_token,
        state.admin_mfa_encryption_key.as_deref(),
    )
    .await?;
    Ok(Json(AdministratorMfaEnrollmentResponse {
        provisioning_uri,
    }))
}

pub async fn confirm_mfa_enrollment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdministratorMfaVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let completed = confirm_administrator_mfa_enrollment(
        &state.database,
        &request.challenge_token,
        &request.code,
        state.admin_mfa_encryption_key.as_deref(),
        get_request_correlation_id(&headers),
    )
    .await?;
    create_authenticated_administrator_response(
        &state,
        completed.session_token,
        completed.administrator_id,
        completed.recovery_codes,
    )
    .await
}

pub async fn verify_mfa_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdministratorMfaVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let completed = verify_administrator_mfa_login(
        &state.database,
        &request.challenge_token,
        &request.code,
        state.admin_mfa_encryption_key.as_deref(),
        get_request_correlation_id(&headers),
    )
    .await?;
    create_authenticated_administrator_response(
        &state,
        completed.session_token,
        completed.administrator_id,
        completed.recovery_codes,
    )
    .await
}

pub async fn regenerate_recovery_codes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Json(request): Json<AdministratorRecoveryCodeRegenerationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let recovery_codes = regenerate_administrator_recovery_codes(
        &state.database,
        identity.administrator_id,
        identity.session_id,
        &request.code,
        state.admin_mfa_encryption_key.as_deref(),
        get_request_correlation_id(&headers),
    )
    .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static(ADMIN_RECOVERY_CODES_NO_STORE_CACHE_CONTROL),
    );
    Ok((
        response_headers,
        Json(AdministratorRecoveryCodesResponse { recovery_codes }),
    ))
}

pub async fn current_administrator(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
) -> Result<impl IntoResponse, AppError> {
    let profile = find_administrator_session(&state.database, identity.administrator_id).await?;
    Ok(Json(AdministratorSessionResponse {
        administrator_id: profile.administrator_id,
        email: profile.email,
        roles: profile.roles,
        permissions: profile.permissions,
        recovery_codes: None,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(identity): Extension<AdministratorSessionIdentity>,
) -> Result<impl IntoResponse, AppError> {
    crate::repositories::admin_auth_repository::revoke_administrator_session(
        &state.database,
        identity.administrator_id,
        identity.session_id,
        get_request_correlation_id(&headers),
    )
    .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        super::auth_cookie::clear_administrator_session_cookie(&state)?,
    );
    Ok((response_headers, axum::http::StatusCode::NO_CONTENT))
}

pub async fn change_own_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Json(request): Json<AdministratorPasswordChangeRequest>,
) -> Result<impl IntoResponse, AppError> {
    change_administrator_password(
        &state.database,
        identity.administrator_id,
        identity.session_id,
        request.current_password,
        request.new_password,
        get_request_correlation_id(&headers),
    )
    .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        super::auth_cookie::clear_administrator_session_cookie(&state)?,
    );
    Ok((response_headers, axum::http::StatusCode::NO_CONTENT))
}

pub async fn current_administrator_sessions(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
) -> Result<impl IntoResponse, AppError> {
    let records = list_administrator_sessions(&state.database, identity.administrator_id).await?;
    let items = records
        .into_iter()
        .map(|record| AdministratorSessionItem {
            id: record.id,
            created_at: record.created_at,
            last_activity_at: record.last_activity_at,
            expires_at: record.expires_at,
            revoked_at: record.revoked_at,
        })
        .collect();
    Ok(Json(AdministratorSessionsResponse { items }))
}

pub async fn show_overview(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let overview = get_administrator_overview(&state.database).await?;
    Ok(Json(AdministratorOverviewResponse {
        registered_accounts: overview.registered_accounts,
        active_accounts: overview.active_accounts,
        pending_email_verification: overview.pending_email_verification,
        suspended_accounts: overview.suspended_accounts,
        total_institutions: overview.total_institutions,
        total_communities: overview.total_communities,
        active_communities: overview.active_communities,
    }))
}

pub async fn revoke_current_administrator_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    axum::extract::Path(session_id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    revoke_administrator_session(
        &state.database,
        identity.administrator_id,
        session_id,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    axum::extract::Query(query): axum::extract::Query<AdministrativeAuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (records, limit, offset) = list_administrative_audit_logs(
        &state.database,
        identity.administrator_id,
        get_request_correlation_id(&headers),
        query.limit,
        query.offset,
    )
    .await?;
    let items = records
        .into_iter()
        .map(|record| AdministrativeAuditLogItem {
            id: record.id,
            actor_administrator_id: record.actor_administrator_id,
            action: record.action,
            target_type: record.target_type,
            target_id: record.target_id,
            result: record.result,
            reason: record.reason,
            correlation_id: record.correlation_id,
            created_at: record.created_at,
        })
        .collect();
    Ok(Json(AdministrativeAuditLogResponse {
        items,
        limit,
        offset,
    }))
}

async fn create_authenticated_administrator_response(
    state: &AppState,
    session_token: String,
    administrator_id: uuid::Uuid,
    recovery_codes: Option<Vec<String>>,
) -> Result<(HeaderMap, Json<AdministratorSessionResponse>), AppError> {
    let profile = find_administrator_session(&state.database, administrator_id).await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        super::auth_cookie::create_administrator_session_cookie(state, &session_token)?,
    );
    Ok((
        response_headers,
        Json(AdministratorSessionResponse {
            administrator_id: profile.administrator_id,
            email: profile.email,
            roles: profile.roles,
            permissions: profile.permissions,
            recovery_codes,
        }),
    ))
}
