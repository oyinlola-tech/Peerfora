use super::support::*;
use axum::http::{StatusCode, header};

pub async fn verify_administrator_sessions_and_permissions(
    context: &TestContext,
    administrator: &AdministratorAuthState,
    account_community: &AccountCommunityFixture,
) {
    let app = context.app.clone();
    let database = context.database.clone();
    let administrator_id = context.administrator_id;
    let administrator_cookie = administrator.administrator_cookie.clone();
    let totp_secret = administrator.totp_secret.clone();
    let code = administrator.code.clone();
    let recovery_code = administrator.recovery_code.clone();
    let student_account_id = account_community.student_account_id;
    let community_id = account_community.community_id;
    let stored_session_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM administrator_sessions
    WHERE administrator_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(administrator_id)
    .fetch_one(&database)
    .await
    .expect("admin session token hash should persist");
    assert_eq!(
        stored_session_hash,
        peerfora_api::utils::one_time_token::hash_one_time_token(&administrator_cookie)
    );
    let encrypted_totp_secret: String =
        sqlx::query_scalar("SELECT mfa_secret_ciphertext FROM administrators WHERE id = $1")
            .bind(administrator_id)
            .fetch_one(&database)
            .await
            .expect("encrypted TOTP secret should persist");
    assert!(!encrypted_totp_secret.contains(&totp_secret));

    let other_session_id = uuid::Uuid::new_v4();
    let other_session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
    VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(other_session_id)
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &other_session_token,
    ))
    .execute(&database)
    .await
    .expect("second session fixture should persist");
    let sessions_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/sessions",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_json = response_json(sessions_response).await;
    assert!(sessions_json["items"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["id"] == other_session_id.to_string())
    }));
    let revoke_other_session = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/admin/sessions/{other_session_id}"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(revoke_other_session.status(), StatusCode::NO_CONTENT);
    let revoked_other_session = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={other_session_token}")),
        None,
    )
    .await;
    assert_eq!(revoked_other_session.status(), StatusCode::UNAUTHORIZED);

    let viewer_token = create_analytics_viewer_test_session(&database, administrator_id).await;
    let viewer_cookie = format!("peerfora_admin_session={viewer_token}");
    let analytics_overview = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/overview",
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(analytics_overview.status(), StatusCode::OK);
    let denied_audit_access = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/audit-logs",
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(denied_audit_access.status(), StatusCode::FORBIDDEN);
    let denied_account_search = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/accounts",
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(denied_account_search.status(), StatusCode::FORBIDDEN);
    let denied_account_suspension = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/accounts/{student_account_id}/suspend"),
        Some(&viewer_cookie),
        Some(serde_json::json!({ "reasonCode": "policy_violation" }).to_string()),
    )
    .await;
    assert_eq!(denied_account_suspension.status(), StatusCode::FORBIDDEN);
    let denied_community_suspension = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/communities/{community_id}/suspend"),
        Some(&viewer_cookie),
        Some(serde_json::json!({ "reasonCode": "policy_violation" }).to_string()),
    )
    .await;
    assert_eq!(denied_community_suspension.status(), StatusCode::FORBIDDEN);
    let administrator_sessions_route =
        peerfora_api::constants::admin_auth::ADMINISTRATOR_SESSIONS_REVOKE_ROUTE
            .replace("{id}", &administrator_id.to_string());
    let denied_administrator_session_revocation = send_request(
        app.clone(),
        "DELETE",
        &administrator_sessions_route,
        Some(&viewer_cookie),
        Some(serde_json::json!({"reason": "security_concern"}).to_string()),
    )
    .await;
    assert_eq!(
        denied_administrator_session_revocation.status(),
        StatusCode::FORBIDDEN
    );
    let audit_access = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/audit-logs?limit=20&offset=0",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(audit_access.status(), StatusCode::OK);
    let audit_json = response_json(audit_access).await;
    assert!(
        audit_json["items"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let second_login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    let second_login_json = response_json(second_login_response).await;
    assert_eq!(second_login_json["nextStep"], "verify_mfa");
    let recovery_login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/verify",
        serde_json::json!({
            "challengeToken": second_login_json["challengeToken"],
            "code": recovery_code
        }),
    )
    .await;
    assert_eq!(recovery_login_response.status(), StatusCode::OK);
    assert!(
        recovery_login_response
            .headers()
            .get(header::SET_COOKIE)
            .is_some()
    );
    let recovery_login_json = response_json(recovery_login_response).await;
    assert!(recovery_login_json.get("recoveryCodes").is_none());

    let reused_recovery_login = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    let reused_recovery_challenge = response_json(reused_recovery_login).await;
    let reused_recovery_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/verify",
        serde_json::json!({
            "challengeToken": reused_recovery_challenge["challengeToken"],
            "code": recovery_code
        }),
    )
    .await;
    assert_eq!(reused_recovery_response.status(), StatusCode::UNAUTHORIZED);

    let third_login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    let third_login_json = response_json(third_login_response).await;
    let replay_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/verify",
        serde_json::json!({
            "challengeToken": third_login_json["challengeToken"],
            "code": code
        }),
    )
    .await;
    assert_eq!(replay_response.status(), StatusCode::UNAUTHORIZED);

    let logout_response = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/auth/logout",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);
    let revoked_session_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(revoked_session_response.status(), StatusCode::UNAUTHORIZED);

    let expired_session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_sessions
        (id, administrator_id, token_hash, created_at, last_activity_at, expires_at)
    VALUES ($1, $2, $3, now() - interval '2 minutes', now(), now() - interval '1 minute')",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &expired_session_token,
    ))
    .execute(&database)
    .await
    .expect("expired session fixture should persist");
    let expired_session_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={expired_session_token}")),
        None,
    )
    .await;
    assert_eq!(expired_session_response.status(), StatusCode::UNAUTHORIZED);

    sqlx::query("UPDATE administrators SET status = 'active' WHERE id = $1")
        .bind(administrator_id)
        .execute(&database)
        .await
        .expect("administrator should be active for MFA lockout fixture");
    for attempt in 0..5 {
        let challenge_hash = peerfora_api::utils::one_time_token::hash_one_time_token(&format!(
            "mfa-lockout-challenge-{attempt}"
        ));
        sqlx::query(
            "INSERT INTO administrator_mfa_challenges
            (token_hash, administrator_id, purpose, expires_at)
        VALUES ($1, $2, 'login', now() + interval '5 minutes')",
        )
        .bind(&challenge_hash)
        .bind(administrator_id)
        .execute(&database)
        .await
        .expect("fresh MFA challenge should persist");
        peerfora_api::repositories::admin_auth_repository::record_failed_administrator_mfa_attempt(
            &database,
            &challenge_hash,
        )
        .await
        .expect("MFA failure should be recorded");
    }
    let mfa_lock: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT mfa_locked_until FROM administrators WHERE id = $1")
            .bind(administrator_id)
            .fetch_one(&database)
            .await
            .expect("administrator MFA lock should be recorded");
    assert!(mfa_lock.is_some_and(|locked_until| locked_until > chrono::Utc::now()));
    let login_during_mfa_lock = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    assert_eq!(login_during_mfa_lock.status(), StatusCode::UNAUTHORIZED);
    let password_attempts_after_mfa_lock: i16 =
        sqlx::query_scalar("SELECT failed_login_attempts FROM administrators WHERE id = $1")
            .bind(administrator_id)
            .fetch_one(&database)
            .await
            .expect("MFA lockout should not count as a password failure");
    assert_eq!(password_attempts_after_mfa_lock, 0);

    let suspended_administrator_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, status, suspended_at)
        VALUES ($1, 'suspended-admin@example.test', 'test-password-hash', 'suspended', now())",
    )
    .bind(suspended_administrator_id)
    .execute(&database)
    .await
    .expect("suspended administrator fixture should persist");
    let suspended_login = send_json(
        app,
        "/api/v1/admin/auth/login",
        serde_json::json!({
            "email": "suspended-admin@example.test",
            "password": "not-the-password"
        }),
    )
    .await;
    assert_eq!(suspended_login.status(), StatusCode::UNAUTHORIZED);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs WHERE actor_administrator_id = $1",
    )
    .bind(administrator_id)
    .fetch_one(&database)
    .await
    .expect("audit rows should persist");
    assert!(audit_count >= 3);
    assert_audit_records_are_append_only(&database).await;
}
