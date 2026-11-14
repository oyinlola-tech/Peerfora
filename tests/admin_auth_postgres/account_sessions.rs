use super::support::*;
use axum::http::StatusCode;
use peerfora_api::{
    constants::admin_auth::{
        ADMIN_ACCOUNT_SESSIONS_REVOKE_ROUTE, ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION,
    },
    enums::account_enforcement_reason::AccountEnforcementReason,
};

pub async fn verify_administrator_session_revocation(
    context: &TestContext,
    administrator: &AdministratorAuthState,
    student_account_id: uuid::Uuid,
) {
    let database = context.database.clone();
    let app = context.app.clone();
    let reason_code = AccountEnforcementReason::AccountCompromise.as_database_value();
    let revocation_route =
        ADMIN_ACCOUNT_SESSIONS_REVOKE_ROUTE.replace("{id}", &student_account_id.to_string());
    let revoked_tokens = [
        peerfora_api::utils::one_time_token::create_one_time_token(),
        peerfora_api::utils::one_time_token::create_one_time_token(),
    ];
    for token in &revoked_tokens {
        sqlx::query(
            "INSERT INTO account_sessions (account_id, token_hash, expires_at)
            VALUES ($1, $2, now() + interval '30 days')",
        )
        .bind(student_account_id)
        .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
            token,
        ))
        .execute(&database)
        .await
        .expect("student session fixture should persist");
    }
    let viewer_token =
        create_analytics_viewer_test_session(&database, context.administrator_id).await;
    let denied_revoke = send_request(
        app.clone(),
        "POST",
        &revocation_route,
        Some(&format!("peerfora_admin_session={viewer_token}")),
        Some(serde_json::json!({ "reasonCode": reason_code }).to_string()),
    )
    .await;
    assert_eq!(denied_revoke.status(), StatusCode::FORBIDDEN);
    let count_before_authorized_revoke: i64 =
        sqlx::query_scalar("SELECT count(*) FROM account_sessions WHERE account_id = $1")
            .bind(student_account_id)
            .fetch_one(&database)
            .await
            .expect("denied access should preserve student sessions");
    assert_eq!(count_before_authorized_revoke, 2);

    let revoke_response = send_request(
        app.clone(),
        "POST",
        &revocation_route,
        Some(&format!(
            "peerfora_admin_session={}",
            administrator.administrator_cookie
        )),
        Some(serde_json::json!({ "reasonCode": reason_code }).to_string()),
    )
    .await;
    assert_eq!(revoke_response.status(), StatusCode::OK);
    let revocation_response_json = response_json(revoke_response).await;
    assert_eq!(revocation_response_json["revokedSessionCount"], 2);

    for token in revoked_tokens {
        let rejected_session = send_request(
            app.clone(),
            "GET",
            "/api/v1/auth/me",
            Some(&format!("peerfora_session={token}")),
            None,
        )
        .await;
        assert_eq!(rejected_session.status(), StatusCode::UNAUTHORIZED);
    }
    let recorded_action: (String, String) = sqlx::query_as(
        "SELECT action, reason FROM administrative_audit_logs
        WHERE actor_administrator_id = $1 AND target_id = $2
            AND action = $3
        ORDER BY created_at DESC LIMIT 1",
    )
    .bind(context.administrator_id)
    .bind(student_account_id)
    .bind(ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION)
    .fetch_one(&database)
    .await
    .expect("session revocation should be audited");
    assert_eq!(
        recorded_action.0,
        ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION
    );
    assert_eq!(recorded_action.1, reason_code);
}
