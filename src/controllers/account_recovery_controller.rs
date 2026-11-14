use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};

use crate::{
    constants::account_auth::{
        EMAIL_CONFIRMED_MESSAGE, MAX_EMAIL_LENGTH, PASSWORD_RESET_COMPLETED_MESSAGE,
        PASSWORD_RESET_REQUESTED_MESSAGE,
    },
    dtos::{
        auth_requests::{
            ConfirmPasswordResetRequest, RequestPasswordResetRequest, VerifyEmailQuery,
        },
        auth_responses::MessageResponse,
    },
    enums::account_auth_error::AccountAuthError,
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::accounts::commands::{
        request_password_reset_command, reset_account_password_command, send_account_email_command,
        verify_account_email_command,
    },
    validations::account_auth_validation::validate_email_address,
};

pub async fn verify_account_email(
    State(state): State<AppState>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<impl IntoResponse, AppError> {
    verify_account_email_command::verify_account_email(&state.database, &query.token).await?;
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok((
        headers,
        Json(MessageResponse {
            message: EMAIL_CONFIRMED_MESSAGE,
        }),
    ))
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(request): Json<RequestPasswordResetRequest>,
) -> Result<impl IntoResponse, AppError> {
    if request.email.len() > MAX_EMAIL_LENGTH {
        return Err(AccountAuthError::InvalidEmail.into());
    }
    validate_email_address(&request.email)?;
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AccountPasswordReset,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    if let Some((email, token)) =
        request_password_reset_command::request_password_reset(&state.database, &request.email)
            .await?
    {
        send_account_email_command::send_password_reset_email(&state, &email, &token).await?;
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse {
            message: PASSWORD_RESET_REQUESTED_MESSAGE,
        }),
    ))
}

pub async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(request): Json<ConfirmPasswordResetRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    reset_account_password_command::reset_account_password(
        &state.database,
        &request.token,
        request.new_password,
    )
    .await?;
    Ok(Json(MessageResponse {
        message: PASSWORD_RESET_COMPLETED_MESSAGE,
    }))
}
