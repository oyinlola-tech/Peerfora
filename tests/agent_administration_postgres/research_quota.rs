use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};

use tower::ServiceExt;
use uuid::Uuid;

use super::{
    create_administrator_session,
    support::{create_account_fixture, response_json, test_app_state},
};

#[tokio::test]
async fn research_submission_quota_requires_permission_and_is_audited() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = peerfora_api::database::connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply migrations");
    let operator = create_account_fixture(&database, "agent_quota").await;
    let app = peerfora_api::app::build_app(test_app_state(database.clone(), database_url));
    let registration = super::send_json_request(
        &app,
        "POST",
        "/api/v1/agents",
        &operator.session_token,
        json!({
            "handle":"quota_managed_agent",
            "displayName":"Quota Managed Agent",
            "requestedCapabilities":["research"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);
    let registration: Value = response_json(registration).await;
    let agent_id = registration["id"].as_str().expect("agent ID");
    let (_, support_token) = create_administrator_session(&database, "support_agent").await;
    let (_, trust_token) = create_administrator_session(&database, "trust_safety_lead").await;
    let path = format!("/api/v1/admin/agents/{agent_id}/research-quota");

    let forbidden = send_quota_update(&app, &path, &support_token, 4).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let invalid = send_quota_update(&app, &path, &trust_token, 0).await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let update = send_quota_update(&app, &path, &trust_token, 1).await;
    assert_eq!(update.status(), StatusCode::NO_CONTENT);

    let (quota, metadata): (i32, Value) = sqlx::query_as(
        "SELECT agent.research_submissions_per_24_hours, audit.metadata
        FROM agent_actor_identities agent
        JOIN administrative_audit_logs audit ON audit.target_id = agent.actor_id
        WHERE agent.actor_id = $1 AND audit.action = 'agents.research_quota.updated'",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("quota and its audit record should persist together");
    assert_eq!(quota, 1);
    assert_eq!(metadata["previousSubmissionsPer24Hours"], 10);
    assert_eq!(metadata["submissionsPer24Hours"], 1);

    let unchanged = send_quota_update(&app, &path, &trust_token, 1).await;
    assert_eq!(unchanged.status(), StatusCode::CONFLICT);
}

async fn send_quota_update(
    app: &axum::Router,
    path: &str,
    token: &str,
    submissions_per_24_hours: i32,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header(header::COOKIE, format!("peerfora_admin_session={token}"))
                .header(header::ORIGIN, super::support::TEST_ORIGIN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"submissionsPer24Hours": submissions_per_24_hours, "reason": "Set an approved agent operating limit."}).to_string(),
                ))
                .expect("quota update request should build"),
        )
        .await
        .expect("router should return a quota update response")
}
