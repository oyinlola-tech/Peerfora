use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    constants::account_auth::REGISTRATION_SUBMITTED_MESSAGE,
    dtos::{auth_requests::RegisterAccountRequest, auth_responses::MessageResponse},
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::accounts::commands::{
        register_account_command::{self, NewAccount},
        send_account_email_command,
    },
    validations::account_auth_validation::{
        parse_date_of_birth, validate_country_code, validate_email_address,
    },
};

pub async fn register_account(
    State(state): State<AppState>,
    Json(request): Json<RegisterAccountRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_email_address(&request.email)?;
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AccountRegistration,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    let date_of_birth = parse_date_of_birth(&request.date_of_birth)?;
    let country_code = validate_country_code(&request.country_code)?;
    let (account, token) = register_account_command::register_account(
        &state.database,
        &state.country_policies,
        NewAccount {
            email: request.email,
            password: request.password,
            username: request.username,
            display_name: request.display_name,
            date_of_birth,
            country_code,
            accepted_terms: request.accepted_terms,
        },
    )
    .await?;
    send_account_email_command::send_account_verification_email(&state, &account.email, &token)
        .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse {
            message: REGISTRATION_SUBMITTED_MESSAGE,
        }),
    ))
}
