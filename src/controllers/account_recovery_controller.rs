use axum::{
    Form, Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse},
};

use crate::{
    constants::account_auth::{
        EMAIL_CONFIRMED_MESSAGE, EMAIL_VERIFICATION_REQUESTED_MESSAGE, MAX_EMAIL_LENGTH,
        PASSWORD_RESET_COMPLETED_MESSAGE, PASSWORD_RESET_REQUESTED_MESSAGE, VERIFY_EMAIL_ROUTE,
    },
    dtos::{
        auth_requests::{
            ConfirmPasswordResetRequest, RequestEmailVerificationRequest,
            RequestPasswordResetRequest, VerifyEmailQuery, VerifyEmailRequest,
        },
        auth_responses::MessageResponse,
    },
    enums::account_auth_error::AccountAuthError,
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::accounts::commands::{
        request_email_verification_command, request_password_reset_command,
        reset_account_password_command, verify_account_email_command,
    },
    validations::account_auth_validation::{validate_email_address, validate_verification_token},
};

pub async fn show_email_verification_confirmation(
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Html<String>, AppError> {
    validate_verification_token(&query.token)?;
    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Confirm email</title></head><body><main><h1>Confirm your email</h1><p>Confirm only if you requested this Peerfora account.</p><form method=\"post\" action=\"{VERIFY_EMAIL_ROUTE}\"><input type=\"hidden\" name=\"token\" value=\"{}\"><button type=\"submit\">Confirm email</button></form></main></body></html>",
        query.token
    )))
}

pub async fn verify_account_email(
    State(state): State<AppState>,
    Form(request): Form<VerifyEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_verification_token(&request.token)?;
    verify_account_email_command::verify_account_email(&state.database, &request.token).await?;
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

pub async fn request_email_verification(
    State(state): State<AppState>,
    Json(request): Json<RequestEmailVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if request.email.len() > MAX_EMAIL_LENGTH {
        return Err(AccountAuthError::InvalidEmail.into());
    }
    validate_email_address(&request.email)?;
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AccountEmailVerification,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    request_email_verification_command::request_email_verification(&state, &request.email).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse {
            message: EMAIL_VERIFICATION_REQUESTED_MESSAGE,
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
    request_password_reset_command::request_password_reset(&state, &request.email).await?;
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
