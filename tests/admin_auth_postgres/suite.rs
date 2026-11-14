mod account_community;
mod account_sessions;
mod admin_security;
mod administrator_lifecycle;
mod administrator_mfa_reset;
mod administrator_password;
mod administrator_password_recovery;
mod authentication;
mod invitations;
mod recovery_codes;
mod role_management;
mod support;

pub async fn run_administrator_postgresql_regression_suite() {
    let Some(context) = support::TestContext::connect_from_environment().await else {
        return;
    };
    let mut administrator = authentication::verify_administrator_login_and_mfa(&context).await;
    let (recovery_codes, current_code) =
        recovery_codes::verify_recovery_code_regeneration(&context, &administrator).await;
    administrator.recovery_code = recovery_codes
        .into_iter()
        .next()
        .expect("regenerated recovery code should be returned");
    administrator.code = current_code;
    let account_community =
        account_community::verify_account_and_community_administration(&context, &administrator)
            .await;
    invitations::verify_administrator_invitations(&context, &administrator).await;
    role_management::verify_administrative_role_management(&context, &administrator).await;
    administrator_lifecycle::verify_administrator_lifecycle_management(&context, &administrator)
        .await;
    account_sessions::verify_administrator_session_revocation(
        &context,
        &administrator,
        account_community.student_account_id,
    )
    .await;
    admin_security::verify_administrator_sessions_and_permissions(
        &context,
        &administrator,
        &account_community,
    )
    .await;
    administrator_mfa_reset::verify_administrator_mfa_reset(&context).await;
    administrator_password::verify_administrator_password_change(&context).await;
    administrator_password_recovery::verify_administrator_password_recovery(&context).await;
    support::verify_readiness_after_database_shutdown(&context).await;
}
