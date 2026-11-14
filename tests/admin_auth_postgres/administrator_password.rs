use super::support::*;
use axum::http::{StatusCode, header};

const ROTATED_ADMINISTRATOR_PASSWORD: &str = "peerfora-admin-password-after-rotation";

pub async fn verify_administrator_password_change(context: &TestContext) {
    let cookie = create_fresh_administrator_session(context).await;
    let route = peerfora_api::constants::admin_auth::ADMIN_AUTH_PASSWORD_CHANGE_ROUTE;
    let unchanged_password = send_request(
        context.app.clone(),
        "POST",
        route,
        Some(&cookie),
        Some(
            serde_json::json!({
                "currentPassword": ADMIN_PASSWORD,
                "newPassword": ADMIN_PASSWORD
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(unchanged_password.status(), StatusCode::BAD_REQUEST);

    let rejected_password = send_request(
        context.app.clone(),
        "POST",
        route,
        Some(&cookie),
        Some(
            serde_json::json!({
                "currentPassword": "incorrect-administrator-password",
                "newPassword": ROTATED_ADMINISTRATOR_PASSWORD
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(rejected_password.status(), StatusCode::UNAUTHORIZED);

    let pending_challenge = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_mfa_challenges (
            token_hash, administrator_id, purpose, expires_at
        ) VALUES ($1, $2, 'login', now() + interval '5 minutes')",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &pending_challenge,
    ))
    .bind(context.administrator_id)
    .execute(&context.database)
    .await
    .expect("pending MFA challenge should persist for password change test");

    let password_change = send_request(
        context.app.clone(),
        "POST",
        route,
        Some(&cookie),
        Some(
            serde_json::json!({
                "currentPassword": ADMIN_PASSWORD,
                "newPassword": ROTATED_ADMINISTRATOR_PASSWORD
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(password_change.status(), StatusCode::NO_CONTENT);
    assert!(
        password_change
            .headers()
            .get(header::SET_COOKIE)
            .is_some_and(|value| value
                .to_str()
                .is_ok_and(|value| value.contains("Max-Age=0")))
    );
    assert_no_active_administrator_sessions(&context.database, context.administrator_id).await;
    assert_pending_challenge_was_consumed(&context.database, &pending_challenge).await;
    assert_password_change_events_were_audited(&context.database, context.administrator_id).await;

    let expired_cookie = send_request(
        context.app.clone(),
        "GET",
        peerfora_api::constants::admin_auth::ADMIN_AUTH_ME_ROUTE,
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(expired_cookie.status(), StatusCode::UNAUTHORIZED);

    let old_password_login = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    assert_eq!(old_password_login.status(), StatusCode::UNAUTHORIZED);
    let new_password_login = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        serde_json::json!({
            "email": ADMIN_EMAIL,
            "password": ROTATED_ADMINISTRATOR_PASSWORD
        }),
    )
    .await;
    assert_eq!(new_password_login.status(), StatusCode::OK);
    assert_eq!(
        response_json(new_password_login).await["nextStep"],
        "verify_mfa"
    );
}

async fn create_fresh_administrator_session(context: &TestContext) -> String {
    let session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(context.administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &session_token,
    ))
    .execute(&context.database)
    .await
    .expect("fresh administrator test session should persist");
    sqlx::query("UPDATE administrators SET mfa_locked_until = NULL WHERE id = $1")
        .bind(context.administrator_id)
        .execute(&context.database)
        .await
        .expect("previous MFA lockout fixture should be cleared");
    format!("peerfora_admin_session={session_token}")
}

async fn assert_no_active_administrator_sessions(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
) {
    let active_session_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrator_sessions
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .fetch_one(database)
    .await
    .expect("administrator session state should be queryable");
    assert_eq!(active_session_count, 0);
}

async fn assert_pending_challenge_was_consumed(database: &sqlx::PgPool, challenge: &str) {
    let is_consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM administrator_mfa_challenges
        WHERE token_hash = $1",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        challenge,
    ))
    .fetch_one(database)
    .await
    .expect("password change should retain challenge history");
    assert!(is_consumed);
}

async fn assert_password_change_events_were_audited(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
) {
    let audit_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM administrative_audit_logs
        WHERE actor_administrator_id = $1
            AND action IN (
                'administrator.password_change_failed',
                'administrator.password_changed'
            )
        ORDER BY created_at, action",
    )
    .bind(administrator_id)
    .fetch_all(database)
    .await
    .expect("password change audit events should persist");
    assert_eq!(
        audit_actions,
        [
            "administrator.password_change_failed",
            "administrator.password_changed"
        ]
    );
}
