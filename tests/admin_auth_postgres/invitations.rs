use super::support::*;
use axum::http::{StatusCode, header};
use peerfora_api::constants::account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER;

pub async fn verify_administrator_invitations(
    context: &TestContext,
    administrator: &AdministratorAuthState,
) {
    let app = context.app.clone();
    let database = context.database.clone();
    let administrator_cookie = administrator.administrator_cookie.clone();
    let sent_emails = context.sent_emails.clone();
    let invitation_response = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/invitations",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(
            serde_json::json!({
                "email": "new-administrator@example.test",
                "roleKey": "support_agent"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(invitation_response.status(), StatusCode::ACCEPTED);
    assert!(
        invitation_response
            .headers()
            .get(header::SET_COOKIE)
            .is_none()
    );
    let email_message = sent_emails
        .lock()
        .expect("recorded invitation email should be readable")
        .last()
        .cloned()
        .expect("invitation email should be delivered");
    let invitation_token = email_message
        .split(&format!("?{ONE_TIME_TOKEN_QUERY_PARAMETER}="))
        .nth(1)
        .and_then(|link| link.split_whitespace().next())
        .map(|token| token.trim_end_matches('.'))
        .expect("invitation token should appear in the recipient email")
        .to_owned();
    assert!(email_message.contains("Support Agent"));
    assert!(email_message.contains("expires in 48 hours"));
    assert!(email_message.contains("complete authenticator setup"));
    assert!(email_message.contains("do not open or forward the link"));
    let stored_invitation_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM administrator_invitations
    WHERE lower(email) = lower('new-administrator@example.test')",
    )
    .fetch_one(&database)
    .await
    .expect("invitation hash should be persisted");
    assert_eq!(
        stored_invitation_hash,
        peerfora_api::utils::one_time_token::hash_one_time_token(&invitation_token)
    );
    assert!(!stored_invitation_hash.contains(&invitation_token));
    let super_administrator_invitation = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/invitations",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(
            serde_json::json!({
                "email": "another-administrator@example.test",
                "roleKey": "super_administrator"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(
        super_administrator_invitation.status(),
        StatusCode::ACCEPTED
    );
    let super_invitation_email = sent_emails
        .lock()
        .expect("Super Admin invitation email should be recorded")
        .last()
        .cloned()
        .expect("Super Admin invitation email should be sent");
    assert!(super_invitation_email.contains("Super Administrator"));
    let super_invitation_token = super_invitation_email
        .split(&format!("?{ONE_TIME_TOKEN_QUERY_PARAMETER}="))
        .nth(1)
        .and_then(|link| link.split_whitespace().next())
        .expect("Super Admin invitation token should appear in the email")
        .trim_end_matches('.')
        .to_owned();
    let super_invitation_role: String = sqlx::query_scalar(
        "SELECT role_key FROM administrator_invitations
        WHERE lower(email) = lower('another-administrator@example.test')",
    )
    .fetch_one(&database)
    .await
    .expect("authorized Super Admin invitation role should persist");
    assert_eq!(super_invitation_role, "super_administrator");
    let super_invitation_acceptance = send_json(
        app.clone(),
        "/api/v1/admin/invitations/accept",
        serde_json::json!({
            "token": super_invitation_token,
            "password": "another-super-admin-password-long-enough"
        }),
    )
    .await;
    assert_eq!(super_invitation_acceptance.status(), StatusCode::CREATED);
    let super_acceptance_json = response_json(super_invitation_acceptance).await;
    assert_eq!(super_acceptance_json["assignedRole"], "super_administrator");
    let super_admin_login = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({
            "email": "another-administrator@example.test",
            "password": "another-super-admin-password-long-enough"
        }),
    )
    .await;
    assert_eq!(super_admin_login.status(), StatusCode::OK);
    assert_eq!(
        response_json(super_admin_login).await["nextStep"],
        "enroll_mfa"
    );
    let first_invitation_acceptance = send_json(
        app.clone(),
        "/api/v1/admin/invitations/accept",
        serde_json::json!({
            "token": invitation_token.clone(),
            "password": "new-admin-password-with-sixteen-bytes"
        }),
    );
    let second_invitation_acceptance = send_json(
        app.clone(),
        "/api/v1/admin/invitations/accept",
        serde_json::json!({
            "token": invitation_token.clone(),
            "password": "new-admin-password-with-sixteen-bytes"
        }),
    );
    let (first_invitation_acceptance, second_invitation_acceptance) =
        tokio::join!(first_invitation_acceptance, second_invitation_acceptance);
    let acceptance_statuses = [
        first_invitation_acceptance.status(),
        second_invitation_acceptance.status(),
    ];
    assert!(acceptance_statuses.contains(&StatusCode::CREATED));
    assert!(acceptance_statuses.contains(&StatusCode::BAD_REQUEST));
    let invitation_acceptance = if first_invitation_acceptance.status() == StatusCode::CREATED {
        first_invitation_acceptance
    } else {
        second_invitation_acceptance
    };
    assert!(
        invitation_acceptance
            .headers()
            .get(header::SET_COOKIE)
            .is_none()
    );
    let acceptance_json = response_json(invitation_acceptance).await;
    assert_eq!(acceptance_json["assignedRole"], "support_agent");
    assert_eq!(acceptance_json["nextStep"], "sign_in_and_enroll_mfa");
    let invited_administrator_id = acceptance_json["administratorId"]
        .as_str()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .expect("activated administrator ID");
    let invited_administrator_login = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({
            "email": "new-administrator@example.test",
            "password": "new-admin-password-with-sixteen-bytes"
        }),
    )
    .await;
    assert_eq!(invited_administrator_login.status(), StatusCode::OK);
    assert!(
        invited_administrator_login
            .headers()
            .get(header::SET_COOKIE)
            .is_none()
    );
    let invited_login_json = response_json(invited_administrator_login).await;
    assert_eq!(invited_login_json["nextStep"], "enroll_mfa");
    let invited_role: String =
        sqlx::query_scalar("SELECT role_key FROM administrator_roles WHERE administrator_id = $1")
            .bind(invited_administrator_id)
            .fetch_one(&database)
            .await
            .expect("the invitation's limited role should be assigned");
    assert_eq!(invited_role, "support_agent");
}
