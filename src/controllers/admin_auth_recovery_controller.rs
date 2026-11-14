use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};

use crate::{
    constants::{
        account_auth::MAX_EMAIL_LENGTH,
        admin_auth::{
            ADMIN_AUTH_NO_REFERRER_POLICY, ADMIN_PASSWORD_RESET_REQUESTED_MESSAGE,
            ADMIN_RECOVERY_CODES_NO_STORE_CACHE_CONTROL,
        },
    },
    dtos::{
        admin_auth::{
            AdministratorPasswordResetConfirmationRequest, AdministratorPasswordResetRequest,
        },
        auth_responses::MessageResponse,
    },
    enums::account_auth_error::AccountAuthError,
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    integrations::email::administrator_password_reset_email::send_administrator_password_reset_email,
    models::app_state::AppState,
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::admin_auth::commands::{
        complete_administrator_password_reset_command::complete_administrator_password_reset,
        request_administrator_password_reset_command::request_administrator_password_reset,
    },
    utils::request_context::get_request_correlation_id,
    validations::account_auth_validation::validate_email_address,
};

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(request): Json<AdministratorPasswordResetRequest>,
) -> Result<impl IntoResponse, AppError> {
    if request.email.len() > MAX_EMAIL_LENGTH {
        return Err(AccountAuthError::InvalidEmail.into());
    }
    validate_email_address(&request.email)?;
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AdministratorPasswordReset,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    if let Some(email_sender) = state.email_sender.as_deref() {
        let reset = request_administrator_password_reset(&state.database, &request.email).await?;
        if let Some(recipient) = reset.recipient {
            let delivery_failed = send_administrator_password_reset_email(
                email_sender,
                &recipient.email,
                &reset.token,
                state.public_url.trim_end_matches('/'),
            )
            .await
            .is_err();
            if delivery_failed {
                tracing::warn!("administrator password reset email delivery failed");
                let cleanup_failed =
                    crate::repositories::admin_auth_repository::revoke_failed_administrator_password_reset_delivery(
                        &state.database,
                        recipient.administrator_id,
                        &reset.token_hash,
                    )
                    .await
                    .is_err();
                if cleanup_failed {
                    tracing::warn!("administrator password reset token cleanup failed");
                }
            }
        }
    }
    Ok((
        StatusCode::ACCEPTED,
        no_store_headers(),
        Json(MessageResponse {
            message: ADMIN_PASSWORD_RESET_REQUESTED_MESSAGE,
        }),
    ))
}

pub async fn confirm_password_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdministratorPasswordResetConfirmationRequest>,
) -> Result<impl IntoResponse, AppError> {
    complete_administrator_password_reset(
        &state.database,
        &request.token,
        request.new_password,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok((no_store_headers(), StatusCode::NO_CONTENT))
}

fn no_store_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(ADMIN_RECOVERY_CODES_NO_STORE_CACHE_CONTROL),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static(ADMIN_AUTH_NO_REFERRER_POLICY),
    );
    headers
}
