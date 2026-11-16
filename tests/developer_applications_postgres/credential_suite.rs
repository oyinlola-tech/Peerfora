use axum::http::StatusCode;
use peerfora_api::constants::developers::DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE;
use peerfora_api::{app::build_app, database::connect_database};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use super::support::{create_account_fixture, response_json, send_json_request, test_app_state};
use super::{create_administrator_session, send_admin_json_request};

#[tokio::test]
async fn approved_developer_credentials_enforce_scope_environment_and_revocation() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let account = create_account_fixture(&database, "developer_credential").await;
    let app = build_app(test_app_state(database.clone(), database_url.clone()));
    let application_id = create_approved_application(&database, &app, &account.session_token).await;
    let (developer_admin_id, developer_admin_token) =
        create_administrator_session(&database, "developer_administrator").await;
    let (_, support_admin_token) = create_administrator_session(&database, "support_agent").await;

    let issued_response =
        issue_credential(&app, &account.session_token, application_id, "First key").await;
    assert_eq!(issued_response.status(), StatusCode::CREATED);
    assert_eq!(issued_response.headers()["cache-control"], "no-store");
    let issued: Value = response_json(issued_response).await;
    let first_credential_id = parse_uuid(&issued["credential"]["id"]);
    let first_secret = issued["secret"].as_str().expect("secret is returned once");
    assert_eq!(issued["credential"]["environment"], "sandbox");
    assert_eq!(issued["credential"]["scopes"][0], "academic_directory_read");

    let credential_list = send_json_request(
        &app,
        "GET",
        &format!("/api/v1/developers/applications/{application_id}/credentials"),
        &account.session_token,
        json!({}),
    )
    .await;
    assert_eq!(credential_list.status(), StatusCode::OK);
    let listed: Value = response_json(credential_list).await;
    assert_eq!(listed["items"][0]["id"], first_credential_id.to_string());
    assert!(listed["items"][0].get("secret").is_none());
    assert!(listed["items"][0].get("tokenHash").is_none());

    let authorized = request_subjects(&app, first_secret).await;
    assert_eq!(authorized.status(), StatusCode::OK);
    assert!(response_json(authorized).await.is_array());
    for _ in 1..DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE {
        assert_eq!(
            request_subjects(&app, first_secret).await.status(),
            StatusCode::OK
        );
    }
    assert_eq!(
        request_subjects(&app, first_secret).await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    let last_used: bool = sqlx::query_scalar(
        "SELECT last_used_at IS NOT NULL FROM developer_application_credentials WHERE id = $1",
    )
    .bind(first_credential_id)
    .fetch_one(&database)
    .await
    .expect("successful credential use should be recorded");
    assert!(last_used);
    assert_eq!(
        request_subjects(&app, &account.session_token)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );

    let mut production_settings = test_app_state(database.clone(), database_url.clone());
    production_settings.production_environment = true;
    let production_app = build_app(production_settings);
    let sandbox_credential_in_production = request_subjects(&production_app, first_secret).await;
    assert_eq!(
        sandbox_credential_in_production.status(),
        StatusCode::UNAUTHORIZED
    );

    set_application_scope_status(&database, application_id, "rejected").await;
    let unapproved_scope = request_subjects(&app, first_secret).await;
    assert_eq!(unapproved_scope.status(), StatusCode::FORBIDDEN);
    set_application_scope_status(&database, application_id, "approved").await;

    let rotated_response = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/credentials/{first_credential_id}/rotate"),
        &account.session_token,
        json!({"expiresInDays":30}),
    )
    .await;
    assert_eq!(rotated_response.status(), StatusCode::CREATED);
    let rotated: Value = response_json(rotated_response).await;
    let rotated_credential_id = parse_uuid(&rotated["credential"]["id"]);
    let rotated_secret = rotated["secret"]
        .as_str()
        .expect("rotated secret appears once");
    assert!(rotated["credential"]["expiresAt"].as_str().is_some());
    assert_eq!(
        request_subjects(&app, first_secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request_subjects(&app, rotated_secret).await.status(),
        StatusCode::OK
    );

    let owner_revoked = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/credentials/{rotated_credential_id}/revoke"),
        &account.session_token,
        json!({}),
    )
    .await;
    assert_eq!(owner_revoked.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        request_subjects(&app, rotated_secret).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let admin_revoked = issue_credential(
        &app,
        &account.session_token,
        application_id,
        "Admin revoked",
    )
    .await;
    assert_eq!(admin_revoked.status(), StatusCode::CREATED);
    let admin_revoked: Value = response_json(admin_revoked).await;
    let admin_revoked_id = parse_uuid(&admin_revoked["credential"]["id"]);
    let admin_revoked_secret = admin_revoked["secret"].as_str().expect("secret exists");
    let forbidden_revoke = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/credentials/{admin_revoked_id}/revoke"),
        &support_admin_token,
        json!({"reason":"Support role cannot revoke machine access."}),
    )
    .await;
    assert_eq!(forbidden_revoke.status(), StatusCode::FORBIDDEN);
    let revoked = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/credentials/{admin_revoked_id}/revoke"),
        &developer_admin_token,
        json!({"reason":"Credential was reported as exposed."}),
    )
    .await;
    assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        request_subjects(&app, admin_revoked_secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let revocation_audit: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM administrative_audit_logs
         WHERE actor_administrator_id = $1 AND target_id = $2
             AND action = 'developers.credential.revoked')",
    )
    .bind(developer_admin_id)
    .bind(admin_revoked_id)
    .fetch_one(&database)
    .await
    .expect("credential revocation should be audited");
    assert!(revocation_audit);

    let suspension_credential = issue_credential(
        &app,
        &account.session_token,
        application_id,
        "Suspended app",
    )
    .await;
    assert_eq!(suspension_credential.status(), StatusCode::CREATED);
    let suspension_credential: Value = response_json(suspension_credential).await;
    let suspension_secret = suspension_credential["secret"]
        .as_str()
        .expect("secret exists");
    let forbidden_suspend = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/suspend"),
        &support_admin_token,
        json!({"reason":"Support role cannot suspend applications."}),
    )
    .await;
    assert_eq!(forbidden_suspend.status(), StatusCode::FORBIDDEN);
    let suspended = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/suspend"),
        &developer_admin_token,
        json!({"reason":"Application violated machine access policy."}),
    )
    .await;
    assert_eq!(suspended.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        request_subjects(&app, suspension_secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let suspension_audit: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM administrative_audit_logs
         WHERE actor_administrator_id = $1 AND target_id = $2
             AND action = 'developers.application.suspended')",
    )
    .bind(developer_admin_id)
    .bind(application_id)
    .fetch_one(&database)
    .await
    .expect("application suspension should be audited");
    assert!(suspension_audit);
}

async fn create_approved_application(
    database: &PgPool,
    app: &axum::Router,
    account_token: &str,
) -> Uuid {
    let profile = send_json_request(
        app,
        "POST",
        "/api/v1/developers/me",
        account_token,
        json!({"displayName":"Credential Tests","description":"Credential suite."}),
    )
    .await;
    assert_eq!(profile.status(), StatusCode::CREATED);
    let registered = send_json_request(
        app,
        "POST",
        "/api/v1/developers/applications",
        account_token,
        json!({"name":"Scoped Directory Tool","applicationType":"server","requestedScopes":["academic_directory_read"]}),
    )
    .await;
    assert_eq!(registered.status(), StatusCode::CREATED);
    let registered: Value = response_json(registered).await;
    let application_id = parse_uuid(&registered["id"]);
    let submitted = send_json_request(
        app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/submit"),
        account_token,
        json!({}),
    )
    .await;
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT);
    let (_, reviewer_token) =
        create_administrator_session(database, "developer_application_reviewer").await;
    let reviewed = send_admin_json_request(
        app,
        &format!("/api/v1/admin/developers/applications/{application_id}/review"),
        &reviewer_token,
        json!({"decision":"approve","reason":"Approved for public academic directory lookup."}),
    )
    .await;
    assert_eq!(reviewed.status(), StatusCode::NO_CONTENT);
    application_id
}

async fn issue_credential(
    app: &axum::Router,
    account_token: &str,
    application_id: Uuid,
    name: &str,
) -> axum::response::Response {
    send_json_request(
        app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/credentials"),
        account_token,
        json!({"name":name}),
    )
    .await
}

async fn request_subjects(app: &axum::Router, credential: &str) -> axum::response::Response {
    use axum::{
        body::Body,
        http::{Request, header},
    };
    use tower::ServiceExt;
    app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/developer-api/subjects?q=mathematics")
                .header(header::AUTHORIZATION, format!("Bearer {credential}"))
                .body(Body::empty())
                .expect("developer API request should build"),
        )
        .await
        .expect("router should return response")
}

async fn set_application_scope_status(database: &PgPool, application_id: Uuid, status: &str) {
    sqlx::query(
        "UPDATE developer_application_requested_scopes SET scope_status = $2
         WHERE application_id = $1 AND scope_key = 'academic.directory.read'",
    )
    .bind(application_id)
    .bind(status)
    .execute(database)
    .await
    .expect("application scope state should be updated");
}

fn parse_uuid(value: &Value) -> Uuid {
    value
        .as_str()
        .expect("identifier should be a string")
        .parse()
        .expect("identifier should be a UUID")
}
