use axum::{Extension, Json, extract::State, http::HeaderMap, response::IntoResponse};

use crate::{
    dtos::admin_invitation::{
        AcceptAdministratorInvitationRequest, AcceptAdministratorInvitationResponse,
        CreateAdministratorInvitationRequest,
    },
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::admin_auth::commands::{
        accept_administrator_invitation_command::accept_administrator_invitation,
        issue_administrator_invitation_command::{
            AdministratorInvitationRequest, issue_administrator_invitation,
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn create_invitation(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    headers: HeaderMap,
    Json(request): Json<CreateAdministratorInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    issue_administrator_invitation(
        &state.database,
        AdministratorInvitationRequest {
            inviter_id: identity.administrator_id,
            inviter_session_id: identity.session_id,
            email: &request.email,
            role_key: &request.role_key,
            public_url: &state.public_url,
            email_sender: state.email_sender.as_deref(),
            correlation_id: get_request_correlation_id(&headers),
        },
    )
    .await?;
    Ok(axum::http::StatusCode::ACCEPTED)
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AcceptAdministratorInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let activated = accept_administrator_invitation(
        &state.database,
        &request.token,
        request.password,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(AcceptAdministratorInvitationResponse {
            administrator_id: activated.administrator_id,
            assigned_role: activated.assigned_role,
            next_step: "sign_in_and_enroll_mfa",
        }),
    ))
}
