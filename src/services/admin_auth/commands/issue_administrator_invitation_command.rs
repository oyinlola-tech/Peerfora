use uuid::Uuid;

use crate::{
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    integrations::email::templates::administrator_invitation_email_template::build_administrator_invitation_email,
    interfaces::email_sender::EmailSender,
    models::admin_invitation::NewAdministratorInvitation,
    repositories::administrator_invitation_repository,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
    validations::account_auth_validation::validate_email_address,
};

pub struct AdministratorInvitationRequest<'a> {
    pub inviter_id: Uuid,
    pub inviter_session_id: Uuid,
    pub email: &'a str,
    pub role_key: &'a str,
    pub public_url: &'a str,
    pub email_sender: Option<&'a dyn EmailSender>,
    pub correlation_id: Option<&'a str>,
}

pub async fn issue_administrator_invitation(
    pool: &sqlx::PgPool,
    request: AdministratorInvitationRequest<'_>,
) -> Result<(), AppError> {
    validate_email_address(request.email).map_err(|_| AccountAuthError::InvalidEmail)?;
    let email_sender = request
        .email_sender
        .ok_or(AccountAuthError::EmailDeliveryNotConfigured)?;
    let normalized_email = normalize_email_address(request.email);
    let invitation_id = Uuid::new_v4();
    let invitation_token = create_one_time_token();
    let invitation_token_hash = hash_one_time_token(&invitation_token);
    let role_display_name = administrator_invitation_repository::create_administrator_invitation(
        pool,
        NewAdministratorInvitation {
            id: invitation_id,
            inviter_id: request.inviter_id,
            email: &normalized_email,
            role_key: request.role_key,
            token_hash: &invitation_token_hash,
        },
        request.inviter_session_id,
        request.correlation_id,
    )
    .await?;
    let invitation_message = build_administrator_invitation_email(
        &normalized_email,
        &role_display_name,
        &invitation_token,
        request.public_url,
    );
    if let Err(email_error) = email_sender
        .send_email(
            &normalized_email,
            invitation_message.subject,
            &invitation_message.plain_text,
        )
        .await
    {
        administrator_invitation_repository::revoke_administrator_invitation_after_email_failure(
            pool,
            invitation_id,
            request.inviter_id,
            request.correlation_id,
        )
        .await?;
        return Err(email_error.into());
    }
    Ok(())
}
