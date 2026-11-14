use super::support::*;
use axum::http::StatusCode;

pub async fn verify_administrator_lifecycle_management(
    context: &TestContext,
    administrator: &AdministratorAuthState,
) {
    let target_administrator_id = uuid::Uuid::new_v4();
    let target_session_id = uuid::Uuid::new_v4();
    let target_session_token = "secondary-admin-session-that-will-be-revoked";
    create_secondary_super_administrator(
        &context.database,
        context.administrator_id,
        target_administrator_id,
        target_session_id,
        target_session_token,
    )
    .await;
    let additional_session_id = uuid::Uuid::new_v4();
    create_additional_administrator_session(
        &context.database,
        target_administrator_id,
        additional_session_id,
        "secondary-admin-second-session",
    )
    .await;
    let pending_mfa_challenge = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_mfa_challenges (
            token_hash, administrator_id, purpose, expires_at
        ) VALUES ($1, $2, 'login', now() + interval '5 minutes')",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &pending_mfa_challenge,
    ))
    .bind(target_administrator_id)
    .execute(&context.database)
    .await
    .expect("pending administrator MFA challenge should persist");
    let administrator_cookie = format!(
        "peerfora_admin_session={}",
        administrator.administrator_cookie
    );
    let session_revocation_path =
        peerfora_api::constants::admin_auth::ADMINISTRATOR_SESSIONS_REVOKE_ROUTE
            .replace("{id}", &target_administrator_id.to_string());
    let session_revocation = send_request(
        context.app.clone(),
        "DELETE",
        &session_revocation_path,
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "security_concern"}).to_string()),
    )
    .await;
    assert_eq!(session_revocation.status(), StatusCode::NO_CONTENT);
    assert_administrator_session_is_revoked(&context.database, target_session_id).await;
    assert_administrator_session_is_revoked(&context.database, additional_session_id).await;
    assert_administrator_mfa_challenge_was_consumed(&context.database, &pending_mfa_challenge)
        .await;
    let revoked_challenge_response = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_MFA_VERIFY_ROUTE,
        serde_json::json!({
            "challengeToken": pending_mfa_challenge,
            "code": "000000"
        }),
    )
    .await;
    assert_eq!(
        revoked_challenge_response.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_administrator_session_revocation_was_audited(
        &context.database,
        context.administrator_id,
        target_administrator_id,
    )
    .await;
    let own_session_revocation_path =
        peerfora_api::constants::admin_auth::ADMINISTRATOR_SESSIONS_REVOKE_ROUTE
            .replace("{id}", &context.administrator_id.to_string());
    let own_session_revocation = send_request(
        context.app.clone(),
        "DELETE",
        &own_session_revocation_path,
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "security_concern"}).to_string()),
    )
    .await;
    assert_eq!(own_session_revocation.status(), StatusCode::CONFLICT);
    let session_created_after_revocation_id = uuid::Uuid::new_v4();
    create_additional_administrator_session(
        &context.database,
        target_administrator_id,
        session_created_after_revocation_id,
        "secondary-admin-session-after-revocation",
    )
    .await;

    let suspension = send_request(
        context.app.clone(),
        "POST",
        &format!("/api/v1/admin/administrators/{target_administrator_id}/suspend"),
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "security_concern"}).to_string()),
    )
    .await;
    assert_eq!(suspension.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        administrator_status(&context.database, target_administrator_id).await,
        "suspended"
    );
    assert_administrator_session_is_revoked(&context.database, target_session_id).await;
    assert_administrator_session_is_revoked(&context.database, session_created_after_revocation_id)
        .await;

    let suspended_login = send_json(
        context.app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({
            "email": "secondary-super-admin@example.test",
            "password": "not-the-password"
        }),
    )
    .await;
    assert_eq!(suspended_login.status(), StatusCode::UNAUTHORIZED);

    let reinstatement = send_request(
        context.app.clone(),
        "POST",
        &format!("/api/v1/admin/administrators/{target_administrator_id}/reinstate"),
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "role_change"}).to_string()),
    )
    .await;
    assert_eq!(reinstatement.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        administrator_status(&context.database, target_administrator_id).await,
        "active"
    );
    assert_administrator_session_is_revoked(&context.database, target_session_id).await;

    let deletion = send_request(
        context.app.clone(),
        "DELETE",
        &format!("/api/v1/admin/administrators/{target_administrator_id}"),
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "departure"}).to_string()),
    )
    .await;
    assert_eq!(deletion.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        administrator_status(&context.database, target_administrator_id).await,
        "deleted"
    );
    assert_deleted_administrator_credentials_are_removed(
        &context.database,
        target_administrator_id,
    )
    .await;
    assert_deleted_administrator_roles_are_revoked(&context.database, target_administrator_id)
        .await;

    let root_suspension = send_request(
        context.app.clone(),
        "POST",
        &format!(
            "/api/v1/admin/administrators/{}/suspend",
            context.administrator_id
        ),
        Some(&administrator_cookie),
        Some(serde_json::json!({"reason": "security_concern"}).to_string()),
    )
    .await;
    assert_eq!(root_suspension.status(), StatusCode::CONFLICT);

    let root_database_protection = sqlx::query(
        "UPDATE administrators SET status = 'suspended'
        WHERE id = $1 AND is_seeded_root",
    )
    .bind(context.administrator_id)
    .execute(&context.database)
    .await;
    assert!(root_database_protection.is_err());
    assert_eq!(
        administrator_status(&context.database, context.administrator_id).await,
        "active"
    );

    let recorded_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM administrative_audit_logs
        WHERE target_id = $1 AND action LIKE 'administrator.%'
        ORDER BY created_at, id",
    )
    .bind(target_administrator_id)
    .fetch_all(&context.database)
    .await
    .expect("administrator lifecycle audit events should persist");
    assert_eq!(
        recorded_actions,
        [
            "administrator.sessions_revoked",
            "administrator.suspended",
            "administrator.login_failed",
            "administrator.reinstated",
            "administrator.deleted"
        ]
    );
}

async fn create_secondary_super_administrator(
    database: &sqlx::PgPool,
    actor_id: uuid::Uuid,
    administrator_id: uuid::Uuid,
    session_id: uuid::Uuid,
    session_token: &str,
) {
    sqlx::query(
        "INSERT INTO administrators (
            id, email, password_hash, status, mfa_secret_ciphertext, mfa_enabled_at
        ) VALUES ($1, 'secondary-super-admin@example.test', 'test-password-hash',
            'active', 'encrypted-test-secret', now())",
    )
    .bind(administrator_id)
    .execute(database)
    .await
    .expect("secondary Super Admin should be created");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
        VALUES ($1, $2, 'super_administrator', $3)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(administrator_id)
    .bind(actor_id)
    .execute(database)
    .await
    .expect("secondary Super Admin role should be assigned");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        session_token,
    ))
    .execute(database)
    .await
    .expect("secondary administrator session should be created");
}

async fn create_additional_administrator_session(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
    session_id: uuid::Uuid,
    session_token: &str,
) {
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        session_token,
    ))
    .execute(database)
    .await
    .expect("additional administrator session should persist");
}

async fn administrator_status(database: &sqlx::PgPool, administrator_id: uuid::Uuid) -> String {
    sqlx::query_scalar("SELECT status FROM administrators WHERE id = $1")
        .bind(administrator_id)
        .fetch_one(database)
        .await
        .expect("administrator status should persist")
}

async fn assert_administrator_session_is_revoked(database: &sqlx::PgPool, session_id: uuid::Uuid) {
    let is_revoked: bool = sqlx::query_scalar(
        "SELECT revoked_at IS NOT NULL FROM administrator_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(database)
    .await
    .expect("administrator session should persist");
    assert!(is_revoked);
}

async fn assert_administrator_session_revocation_was_audited(
    database: &sqlx::PgPool,
    actor_id: uuid::Uuid,
    target_id: uuid::Uuid,
) {
    let was_audited: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrative_audit_logs
            WHERE actor_administrator_id = $1 AND target_id = $2
                AND action = 'administrator.sessions_revoked'
                AND result = 'success'
        )",
    )
    .bind(actor_id)
    .bind(target_id)
    .fetch_one(database)
    .await
    .expect("administrator session revocation audit should persist");
    assert!(was_audited);
}

async fn assert_administrator_mfa_challenge_was_consumed(
    database: &sqlx::PgPool,
    challenge_token: &str,
) {
    let is_consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM administrator_mfa_challenges
        WHERE token_hash = $1",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        challenge_token,
    ))
    .fetch_one(database)
    .await
    .expect("administrator MFA challenge should remain auditable");
    assert!(is_consumed);
}

async fn assert_deleted_administrator_credentials_are_removed(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
) {
    let (email, password_hash, mfa_secret_ciphertext, mfa_enabled_at): (
        String,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        "SELECT email, password_hash, mfa_secret_ciphertext, mfa_enabled_at
        FROM administrators WHERE id = $1",
    )
    .bind(administrator_id)
    .fetch_one(database)
    .await
    .expect("deleted administrator record should remain for audit history");
    assert!(email.ends_with("@invalid.peerfora"));
    assert!(password_hash.is_none());
    assert!(mfa_secret_ciphertext.is_none());
    assert!(mfa_enabled_at.is_none());
}

async fn assert_deleted_administrator_roles_are_revoked(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
) {
    let active_role_assignments: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrator_roles
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .fetch_one(database)
    .await
    .expect("administrator role assignment state should be queryable");
    assert_eq!(active_role_assignments, 0);
}
