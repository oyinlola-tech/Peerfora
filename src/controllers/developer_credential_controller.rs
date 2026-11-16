use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    constants::developers,
    controllers::research_auth::require_authenticated_account_id,
    dtos::developer::{
        AdministrativeDeveloperCredentialRevocationRequest, DeveloperApplicationEnforcementRequest,
        DeveloperCredentialItem, DeveloperCredentialListResponse, IssueDeveloperCredentialRequest,
        IssuedDeveloperCredentialResponse, RotateDeveloperCredentialRequest,
    },
    enums::developer::DeveloperCredentialEnvironment,
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::developers::commands::{
        administrate_developer_application_command, issue_developer_credential_command,
        revoke_developer_credential_command, rotate_developer_credential_command,
    },
    services::developers::queries::list_developer_credentials_query,
    utils::request_context::get_request_correlation_id,
};

pub async fn issue_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(application_id): Path<Uuid>,
    Json(request): Json<IssueDeveloperCredentialRequest>,
) -> Result<Response, AppError> {
    let owner_account_id = require_authenticated_account_id(&state, &headers).await?;
    let issued = issue_developer_credential_command::issue_developer_credential(
        &state.database,
        owner_account_id,
        application_id,
        &request.name,
        request.expires_in_days,
        get_runtime_credential_environment(state.production_environment),
    )
    .await?;
    let response = IssuedDeveloperCredentialResponse {
        credential: issued.record.try_into()?,
        secret: issued.secret,
    };
    Ok(secret_response(StatusCode::CREATED, response))
}

pub async fn list_credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(application_id): Path<Uuid>,
) -> Result<Json<DeveloperCredentialListResponse>, AppError> {
    let owner_account_id = require_authenticated_account_id(&state, &headers).await?;
    let records = list_developer_credentials_query::list_developer_credentials(
        &state.database,
        owner_account_id,
        application_id,
    )
    .await?;
    let items = records
        .into_iter()
        .map(DeveloperCredentialItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(DeveloperCredentialListResponse { items }))
}

pub async fn revoke_own_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((application_id, credential_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let owner_account_id = require_authenticated_account_id(&state, &headers).await?;
    revoke_developer_credential_command::revoke_developer_credential(
        &state.database,
        owner_account_id,
        application_id,
        credential_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn rotate_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((application_id, credential_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<RotateDeveloperCredentialRequest>,
) -> Result<Response, AppError> {
    let owner_account_id = require_authenticated_account_id(&state, &headers).await?;
    let rotated = rotate_developer_credential_command::rotate_developer_credential(
        &state.database,
        owner_account_id,
        application_id,
        credential_id,
        request.expires_in_days,
        get_runtime_credential_environment(state.production_environment),
    )
    .await?;
    let response = IssuedDeveloperCredentialResponse {
        credential: rotated.record.try_into()?,
        secret: rotated.secret,
    };
    Ok(secret_response(StatusCode::CREATED, response))
}

pub async fn suspend_application(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(application_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<DeveloperApplicationEnforcementRequest>,
) -> Result<StatusCode, AppError> {
    administrate_developer_application_command::suspend_developer_application(
        &state.database,
        application_id,
        identity.administrator_id,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_credential_as_administrator(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path((application_id, credential_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(request): Json<AdministrativeDeveloperCredentialRevocationRequest>,
) -> Result<StatusCode, AppError> {
    administrate_developer_application_command::revoke_developer_credential_as_administrator(
        &state.database,
        application_id,
        credential_id,
        identity.administrator_id,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn get_runtime_credential_environment(
    production_environment: bool,
) -> DeveloperCredentialEnvironment {
    if production_environment {
        DeveloperCredentialEnvironment::Production
    } else {
        DeveloperCredentialEnvironment::Sandbox
    }
}

fn secret_response<T: serde::Serialize>(status: StatusCode, body: T) -> Response {
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(developers::DEVELOPER_CREDENTIAL_SECRET_CACHE_CONTROL),
    );
    response
}
